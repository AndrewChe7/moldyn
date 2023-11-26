use std::collections::{BTreeMap, HashMap};
use std::fs::{File, OpenOptions};
use std::io::BufWriter;
use std::path::Path;
use itertools::Itertools;
use na::Vector3;
use serde::{Deserialize, Serialize, Serializer};
use crate::{Particle, ParticleDatabase, State};

/// Serialization struct for [Particle]
#[derive(Serialize, Deserialize, Clone)]
pub struct ParticleToSave {
    pub id: u16,
    pub position_x: f64,
    pub position_y: f64,
    pub position_z: f64,
    pub velocity_x: f64,
    pub velocity_y: f64,
    pub velocity_z: f64,
}

/// Serialization struct for [State]
#[derive(Serialize, Deserialize, Clone)]
pub struct StateToSave {
    pub particles: Vec<ParticleToSave>,
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

/// Structure for serialization of macro parameters
#[derive(Serialize, Deserialize, Clone)]
pub struct MacroParameters {
    pub iteration: usize,
    pub kinetic_energy: f64,
    pub potential_energy: f64,
    pub thermal_energy: f64,
    pub unit_kinetic_energy: f64,
    pub unit_potential_energy: f64,
    pub unit_thermal_energy: f64,
    pub temperature: f64,
    pub pressure: f64,
    pub custom: f64,
}

/// Structure to serialize macro parameters
#[derive(Serialize, Deserialize, Clone)]
pub struct DataFileMacro {
    #[serde(serialize_with = "ordered_map")]
    pub macro_parameters: HashMap<usize, MacroParameters>,
    pub start_frame: usize,
    pub frame_count: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VectorData {
    pub x: f64,
    pub y: f64,
    pub z: f64,
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
        mass?;
        let mass = mass.unwrap();
        let radius = ParticleDatabase::get_particle_radius(id).unwrap();
        Some(
            Particle {
                position: Vector3::new(self.position_x, self.position_y, self.position_z),
                velocity: Vector3::new(self.velocity_x, self.velocity_y, self.velocity_z),
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
            position_x: particle.position.x,
            position_y: particle.position.y,
            position_z: particle.position.z,
            velocity_x: particle.velocity.x,
            velocity_y: particle.velocity.y,
            velocity_z: particle.velocity.z,
            id: particle.id,
        }
    }
}

impl From<&State> for StateToSave {
    fn from(state: &State) -> Self {
        let boundary_box = state.boundary_box;
        let mut particles: Vec<ParticleToSave> = vec![];
        state.particles.iter().for_each(|t| {
            t.iter().for_each(|particle| {
                particles.push(ParticleToSave::from(particle));
            });
        });
        Self {
            particles,
            boundary_box,
        }
    }
}

impl Into<State> for StateToSave {
    fn into(self) -> State {
        let mut particles = vec![];
        let mut max_id = 0;
        for particle in self.particles.iter() {
            if particle.id > max_id {
                max_id = particle.id;
            }
        }
        for _ in 0..=max_id {
            particles.push(vec![]);
        }
        for particle in self.particles.iter() {
            let particle: Particle = particle.into().expect("Can't convert particle");
            particles[particle.id as usize].push(particle);
        }
        let boundary_box = self.boundary_box;
        State {
            particles,
            boundary_box,
        }
    }
}

impl StateToSave {

    fn get_bbs(path: &Path) -> Vec<Vector3<f64>> {
        let mut bbs: Vec<Vector3<f64>> = vec![];
        let mut reader = csv::Reader::from_path(path).expect("Can't open file");
        for data in reader.deserialize() {
            let bb: VectorData = data.expect("Can't deserialize");
            bbs.push(Vector3::new(bb.x, bb.y, bb.z));
        }
        bbs
    }

    fn save_bb(&self, path: &Path, state_number: usize) {
        let bb_file = if !path.exists() {
            File::create(path)
        } else {
            OpenOptions::new().write(true).open(path)
        }.expect("Can't write to file");
        let mut bbs = Self::get_bbs(path);
        if bbs.len() > state_number {
            bbs[state_number] = self.boundary_box;
        } else {
            bbs.push(self.boundary_box);
        }
        let buf_writer = BufWriter::with_capacity(1073741824, bb_file);
        let mut wtr = csv::Writer::from_writer(buf_writer);
        for value in bbs {
            wtr.serialize(VectorData { x: value.x, y: value.y, z: value.z })
                .expect("Can't serialize data");
        }
        wtr.flush().expect("Can't write");
    }

    fn load_bb(path:&Path, state_number: usize) -> Vector3<f64> {
        let bbs = Self::get_bbs(path);
        let bb = bbs[state_number];
        bb
    }

    pub fn save_to_file(&self, path: &Path, state_number: usize) {
        if !path.is_dir() {
            std::fs::create_dir_all(path).expect(format!("Can't create directory in {}",
                                                         path.to_str().unwrap()).as_str());
        }

        let bb_path = path.join("bb.csv");
        self.save_bb(&bb_path, state_number);
        let path = path.join("data");
        if !path.is_dir() {
            std::fs::create_dir_all(&path).expect(format!("Can't create directory in {}",
                                                         path.to_str().unwrap()).as_str());
        }
        let path = path.join(format!("{state_number}.csv"));
        let file = if !path.exists() {
            File::create(path)
        } else {
            OpenOptions::new().truncate(true).write(true).open(path)
        }.expect("Can't write to file");
        let buf_writer = BufWriter::with_capacity(1073741824, file);
        let mut wtr = csv::Writer::from_writer(buf_writer);
        for value in self.particles.iter() {
            wtr.serialize(value.clone()).expect("Can't serialize data");
        }
        wtr.flush().expect("Can't write");
    }

    pub fn load_from_file(path: &Path, state_number: usize) -> Self {
        let bb_path = path.join("bb.csv");
        let bb = Self::load_bb(&bb_path, state_number);
        let path = path.join("data").join(format!("{state_number}.csv"));
        let mut reader = csv::Reader::from_path(path).expect("Can't open file");
        let mut particles = vec![];
        for data in reader.deserialize() {
            let data: ParticleToSave = data.expect("Can't parse row");
            particles.push(data);
        }
        Self {
            particles,
            boundary_box: bb,
        }
    }
}

impl DataFileMacro {
    /// Creates empty structure
    pub fn new() -> Self {
        Self {
            macro_parameters: HashMap::new(),
            start_frame: 0,
            frame_count: 0,
        }
    }

    /// Add multiple macro parameters
    ///
    /// # Arguments
    /// * `frame` - frame number to add macro parameter
    /// * `parameters` - slice of [MacroParameterType] each keeps data to save
    /// * `particles_count` - amount of particles in system
    ///
    pub fn add_macro_params(&mut self, frame: usize, parameters: &[MacroParameterType], particles_count: usize) {
        let macro_parameters = self.macro_parameters.entry(frame).or_insert(MacroParameters {
            iteration: frame,
            kinetic_energy: 0.0,
            potential_energy: 0.0,
            thermal_energy: 0.0,
            unit_kinetic_energy: 0.0,
            unit_potential_energy: 0.0,
            unit_thermal_energy: 0.0,
            temperature: 0.0,
            pressure: 0.0,
            custom: 0.0,
        });
        for parameter in parameters {
            match parameter {
                MacroParameterType::KineticEnergy(value) => {
                    macro_parameters.kinetic_energy = *value;
                    macro_parameters.unit_kinetic_energy = *value / particles_count as f64;
                }
                MacroParameterType::PotentialEnergy(value) => {
                    macro_parameters.potential_energy = *value;
                    macro_parameters.unit_potential_energy = *value / particles_count as f64;
                }
                MacroParameterType::ThermalEnergy(value) => {
                    macro_parameters.thermal_energy = *value;
                    macro_parameters.unit_thermal_energy = *value / particles_count as f64;
                }
                MacroParameterType::Pressure(value) => {
                    macro_parameters.pressure = *value;
                }
                MacroParameterType::Temperature(value) => {
                    macro_parameters.temperature = *value;
                }
                MacroParameterType::Custom(_id, _value) => {
                    todo!()
                }
            }
        }
        self.frame_count += 1;
    }

    /// Save all macro parameters to file. It uses CSV format.
    pub fn save_to_file (&self, path: &Path) {
        let file = if !path.exists() {
            File::create(path)
        } else {
            OpenOptions::new().truncate(true).write(true).open(path)
        }.expect("Can't write to file");
        let buf_writer = BufWriter::with_capacity(1073741824, file);
        let mut wtr = csv::Writer::from_writer(buf_writer);
        for (_key, value) in self.macro_parameters.iter()
            .sorted_by_key(|x| *x.0) {
            wtr.serialize(value.clone()).expect("Can't serialize data");
        }
        wtr.flush().expect("Can't write");
    }

    /// Removes old frames. This function is used when you want to keep your data in separate files.
    pub fn reset_old(&mut self) {
        self.macro_parameters.clear();
        self.start_frame += self.frame_count;
        self.frame_count = 0;
    }

    /// Extends one macro parameters structure with another.
    /// > **Warning**
    /// > After this function `start_frame` and `frame_count` become incorrect.
    pub fn append_data(&mut self, another: &DataFileMacro) {
        self.macro_parameters.extend(another.macro_parameters.clone());
    }

    /// Loads from CSV file macro parameters data.
    pub fn load_from_file (path: &Path) -> Self {
        let mut reader = csv::Reader::from_path(path).expect("Can't open file");
        let mut macro_parameters = HashMap::new();
        let mut min_iter = usize::MAX;
        let mut max_iter = 0;
        for data in reader.deserialize() {
            let data: MacroParameters = data.expect("Can't parse row");
            if data.iteration > max_iter {
                max_iter = data.iteration;
            }
            if data.iteration < min_iter {
                min_iter = data.iteration;
            }
            let _ = macro_parameters.insert(data.iteration, data);
        }
        Self {
            macro_parameters,
            start_frame: min_iter,
            frame_count: max_iter - min_iter,
        }
    }
}

impl Default for DataFileMacro {
    fn default() -> Self {
        Self::new()
    }
}
