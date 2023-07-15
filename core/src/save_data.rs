use std::collections::{BTreeMap, HashMap};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::sync::RwLock;
use na::Vector3;
use serde::{Deserialize, Serialize, Serializer};
use crate::{Particle, ParticleDatabase, State};
use crate::particles_database::ParticleData;

#[derive(Serialize, Deserialize, Clone)]
pub struct ParticleToSave {
    /// position of particle in 3d space
    pub position: Vector3<f64>,
    /// velocity of particle
    pub velocity: Vector3<f64>,
    /// ID of particle. Defines type of particle
    pub id: u16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StateToSave {
    #[serde(serialize_with = "ordered_map")]
    pub particles: HashMap<usize, ParticleToSave>,
    pub boundary_box: Vector3<f64>,
}

pub enum MacroParameterType {
    KineticEnergy(f64),
    PotentialEnergy(f64),
    ThermalEnergy(f64),
    Temperature(f64),
    Pressure(f64),
    Custom(usize, f64),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MacroParameters {
    pub kinetic_energy: f64,
    pub potential_energy: f64,
    pub thermal_energy: f64,
    pub unit_kinetic_energy: f64,
    pub unit_potential_energy: f64,
    pub unit_thermal_energy: f64,
    pub temperature: f64,
    pub pressure: f64,
    pub custom: HashMap<usize, f64>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DataFile {
    #[serde(serialize_with = "ordered_map")]
    pub frames: HashMap<usize, StateToSave>,
    #[serde(serialize_with = "ordered_map")]
    pub particles_database: HashMap<u16, ParticleData>,
    #[serde(serialize_with = "ordered_map")]
    pub macro_parameters: HashMap<usize, MacroParameters>,
    pub start_frame: usize,
    pub frame_count: usize,
}

fn ordered_map<S, K, V>(value: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        K: Serialize + Ord,
        V: Serialize,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}

impl ParticleToSave {
    pub fn into(&self) -> Option<Particle> {
        let id = self.id;
        let mass = ParticleDatabase::get_particle_mass(id);
        if mass.is_none() {
            return None;
        }
        let mass = mass.unwrap();
        let radius = ParticleDatabase::get_particle_radius(id).unwrap();
        Some(
            Particle {
                position: self.position,
                velocity: self.velocity,
                force: Default::default(),
                potential: 0.0,
                temp: 0.0,
                mass,
                radius,
                id,
            }
        )
    }

    pub fn from(particle: &Particle) -> ParticleToSave {
        ParticleToSave {
            position: particle.position.clone(),
            velocity: particle.velocity.clone(),
            id: particle.id,
        }
    }
}

impl StateToSave {
    pub fn from(state: &State) -> Self {
        let mut particles: HashMap<usize, ParticleToSave> = HashMap::new();
        let boundary_box = state.boundary_box.clone();
        for (i, particle) in state.particles.iter().enumerate() {
            let particle = particle.read().expect("Can't lock particle");
            let particle = ParticleToSave {
                position: particle.position.clone(),
                velocity: particle.velocity.clone(),
                id: particle.id.clone(),
            };
            particles.insert(i, particle);
        }
        Self {
            particles,
            boundary_box,
        }
    }

    pub fn into(&self) -> State {
        let particle_count = self.particles.len();
        let mut particles = vec![];
        for _ in 0..particle_count {
            particles.push(RwLock::new(Particle::default()));
        }
        let boundary_box = self.boundary_box.clone();
        for (i, particle) in self.particles.iter() {
            let particle: Particle = particle.into().expect("No particle in database");
            particles[i.clone()] = RwLock::new(particle);
        }
        State {
            particles,
            boundary_box,
        }
    }
}

impl DataFile {
    pub fn init_from_state(state: &State) -> DataFile {
        let particles_database = {
            let particle_database = ParticleDatabase::get_data();
            let hash_table = particle_database.read()
                .expect("Can't lock particles database");
            hash_table.clone()
        };
        let mut frames = HashMap::new();
        let state = StateToSave::from(state);
        frames.insert(0, state);
        DataFile {
            frames,
            particles_database,
            macro_parameters: HashMap::new(),
            start_frame: 0,
            frame_count: 1,
        }
    }

    pub fn add_state(&mut self, state: &State) {
        let state = StateToSave::from(state);
        let last_frame = self.start_frame + self.frame_count;
        self.frames.insert(last_frame, state);
        self.frame_count += 1;
    }

    pub fn add_macro_params(&mut self, frame: usize, parameters: &[MacroParameterType], particles_count: usize) {
        if !self.frames.contains_key(&frame) {
            return;
        }
        let mut macro_parameters = self.macro_parameters.entry(frame).or_insert(MacroParameters {
            kinetic_energy: 0.0,
            potential_energy: 0.0,
            thermal_energy: 0.0,
            unit_kinetic_energy: 0.0,
            unit_potential_energy: 0.0,
            unit_thermal_energy: 0.0,
            temperature: 0.0,
            pressure: 0.0,
            custom: HashMap::new(),
        });
        for parameter in parameters {
            match parameter {
                MacroParameterType::KineticEnergy(value) => {
                    macro_parameters.kinetic_energy = value.clone();
                    macro_parameters.unit_kinetic_energy = value.clone() / particles_count as f64;
                }
                MacroParameterType::PotentialEnergy(value) => {
                    macro_parameters.potential_energy = value.clone();
                    macro_parameters.unit_potential_energy = value.clone() / particles_count as f64;
                }
                MacroParameterType::ThermalEnergy(value) => {
                    macro_parameters.thermal_energy = value.clone();
                    macro_parameters.unit_thermal_energy = value.clone() / particles_count as f64;
                }
                MacroParameterType::Pressure(value) => {
                    macro_parameters.pressure = value.clone();
                }
                MacroParameterType::Temperature(value) => {
                    macro_parameters.temperature = value.clone();
                }
                MacroParameterType::Custom(_id, _value) => {
                    todo!()
                }
            }
        }
    }

    pub fn merge_data_files(a: &DataFile, b: &DataFile) -> Result<DataFile, ()> {
        if a.start_frame + a.frame_count != b.start_frame {
            return Err(());
        }
        if a.frames.get(&a.start_frame).unwrap().particles.len() !=
            b.frames.get(&b.start_frame).unwrap().particles.len() {
            return Err(());
        }
        let mut particles_database = a.particles_database.clone();
        particles_database.extend(b.particles_database.clone());
        let mut frames = a.frames.clone();
        frames.extend(b.frames.clone());
        let mut macro_parameters = a.macro_parameters.clone();
        macro_parameters.extend(b.macro_parameters.clone());
        let result = DataFile {
            frames,
            particles_database,
            macro_parameters,
            start_frame: a.start_frame,
            frame_count: a.frame_count + b.frame_count,
        };
        Ok(result)
    }

    pub fn split_data_file(data: &DataFile, frame_count_for_first_part: usize) -> Result<(DataFile, DataFile), ()> {
        if frame_count_for_first_part > data.frame_count {
            return Err(());
        }
        let mut a = HashMap::new();
        let mut b = HashMap::new();
        let mut a_macro = HashMap::new();
        let mut b_macro = HashMap::new();
        for (i, frame) in &data.frames {
            let t = *i - data.start_frame;
            if t < frame_count_for_first_part {
                a.insert(i.clone(), frame.clone());
            } else {
                b.insert(i.clone(), frame.clone());
            }
        }
        for (i, macro_param) in &data.macro_parameters {
            let t = *i - data.start_frame;
            if t < frame_count_for_first_part {
                a_macro.insert(i.clone(), macro_param.clone());
            } else {
                b_macro.insert(i.clone(), macro_param.clone());
            }
        }
        let left = DataFile {
            frames: a,
            particles_database: data.particles_database.clone(),
            macro_parameters: a_macro,
            start_frame: data.start_frame,
            frame_count: frame_count_for_first_part,
        };
        let right = DataFile {
            frames: b,
            particles_database: data.particles_database.clone(),
            macro_parameters: b_macro,
            start_frame: data.start_frame + frame_count_for_first_part,
            frame_count: data.frame_count - frame_count_for_first_part,
        };
        Ok((left, right))
    }

    pub fn save_to_file (&self, path: &Path, pretty_print: bool) {
        let file = if !path.exists() {
            File::create(path)
        } else {
            OpenOptions::new().truncate(true).write(true).open(path)
        }.expect("Can't write to file");
        let mut buf_writer = BufWriter::new(file);
        if pretty_print {
            serde_json::ser::to_writer_pretty(&mut buf_writer, &self)
                .expect("Can't save data to file");
        } else {
            serde_json::ser::to_writer(&mut buf_writer, &self)
                .expect("Can't save data to file");
        }
    }

    pub fn load_from_file (path: &Path) -> Self {
        let file = File::open(path).expect("Can't open file to read");
        let buf_reader = BufReader::new(file);
        let data_file: Self = serde_json::de::from_reader(buf_reader).expect("Can't read file");
        data_file
    }

    pub fn get_last_frame(&self) -> State {
        let last_frame_number = self.start_frame + self.frame_count - 1;
        let last_frame = self.frames.get(&last_frame_number)
            .expect("Can't get frame");
        last_frame.into()
    }
}
