use std::collections::{BTreeMap, HashMap};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::Path;
use itertools::Itertools;
use na::Vector3;
use serde::{Deserialize, Serialize, Serializer};
use crate::{Particle, ParticleDatabase, State, Structure};
use crate::particles_database::ParticleData;

/// Serialization struct for [Particle]
#[derive(Serialize, Deserialize, Clone)]
pub struct ParticleToSave {
    pub position: Vector3<f64>,
    pub velocity: Vector3<f64>,
    pub id: u16,
}

/// Serialization struct for [State]
#[derive(Serialize, Deserialize, Clone)]
pub struct StateToSave {
    #[serde(serialize_with = "ordered_map")]
    pub particles: HashMap<u16, HashMap<usize, ParticleToSave>>,
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

/// Structure to serialize animation data for particles
#[derive(Serialize, Deserialize, Clone)]
pub struct DataFile {
    /// Each frame is state with particles and boundary conditions
    #[serde(serialize_with = "ordered_map")]
    pub frames: HashMap<usize, StateToSave>,
    /// Particles that was in use in simulation
    #[serde(serialize_with = "ordered_map")]
    pub particles_database: HashMap<u16, ParticleData>,
    /// First frame in file
    pub start_frame: usize,
    /// Count of frames in simulation
    pub frame_count: usize,
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
pub struct DataFileStructure {
    #[serde(serialize_with = "ordered_map")]
    pub particles: HashMap<u16, HashMap<usize, ParticleToSave>>,
    pub origin: [f64; 3],
    pub particles_database: HashMap<u16, ParticleData>,
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
            position: particle.position,
            velocity: particle.velocity,
            id: particle.id,
        }
    }
}

impl StateToSave {
    pub fn from(state: &State) -> Self {
        let mut particles: HashMap<u16, HashMap<usize, ParticleToSave>> = HashMap::new();
        let boundary_box = state.boundary_box;
        for (id, particle_type) in state.particles.iter().enumerate() {
            let id = id as u16;
            particles.insert(id, HashMap::new());

            for (i, particle) in particle_type.iter().enumerate() {
                let particle = ParticleToSave {
                    position: particle.position,
                    velocity: particle.velocity,
                    id: particle.id,
                };
                particles.get_mut(&id).unwrap().insert(i, particle);
            }
        }
        Self {
            particles,
            boundary_box,
        }
    }

    pub fn into(&self) -> State {
        let particle_type_count = self.particles.len();
        let mut particles = vec![];
        for particle_type in 0..particle_type_count {
            particles.push(vec![]);
            let particle_type = particle_type as u16;
            let particles_with_type = self.particles.get(&particle_type)
                .expect("No particle type");
            let particles_count = particles_with_type.len();
            for _ in 0..particles_count {
                particles[particle_type as usize].push(Particle::default());
            }
        }
        let boundary_box = self.boundary_box;
        for (id, particle_type) in self.particles.iter() {
            for (i, particle) in particle_type {
                let particle: Particle = particle.into().expect("No particle in database");
                particles[*id as usize][*i] = particle;
            }
        }
        State {
            particles,
            boundary_box,
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

impl DataFile {
    /// Creates new data file structure from existing state
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
            start_frame: 0,
            frame_count: 1,
        }
    }

    /// Add new state to data file structure
    pub fn add_state(&mut self, state: &State) {
        let state = StateToSave::from(state);
        let last_frame = self.start_frame + self.frame_count;
        self.frames.insert(last_frame, state);
        self.frame_count += 1;
    }

    /// Save all data to file
    pub fn save_to_file (&self, path: &Path, pretty_print: bool) {
        let file = if !path.exists() {
            File::create(path)
        } else {
            OpenOptions::new().truncate(true).write(true).open(path)
        }.expect("Can't write to file");
        let mut buf_writer = BufWriter::with_capacity(1073741824, file);
        if pretty_print {
            serde_json::ser::to_writer_pretty(&mut buf_writer, &self)
                .expect("Can't save data to file");
        } else {
            serde_json::ser::to_writer(&mut buf_writer, &self)
                .expect("Can't save data to file");
        }
    }

    /// Removes old frames. This function is used when you want to keep your data in separate files.
    pub fn reset_old(&mut self) {
        self.frames.clear();
        self.start_frame += self.frame_count;
        self.frame_count = 0;
    }

    /// Load data file structure from file
    pub fn load_from_file (path: &Path) -> Self {
        let file = File::open(path).expect("Can't open file to read");
        let buf_reader = BufReader::with_capacity(1073741824, file);
        let data_file: Self = serde_json::de::from_reader(buf_reader).expect("Can't read file");
        data_file
    }

    /// Get last frame state from data file structure
    pub fn get_last_frame(&self) -> (usize, State) {
        let last_frame_number = self.start_frame + self.frame_count - 1;
        let last_frame = self.frames.get(&last_frame_number)
            .expect("Can't get frame");
        (last_frame_number, last_frame.into())
    }
}

impl DataFileStructure {
    fn from(structure: &Structure) -> Self {
        let particles_database = {
            let particle_database = ParticleDatabase::get_data();
            let hash_table = particle_database.read()
                .expect("Can't lock particles database");
            hash_table.clone()
        };
        let mut particles: HashMap<u16, HashMap<usize, ParticleToSave>> = HashMap::new();
        for (id, particle_type) in structure.particles.iter().enumerate() {
            let id = id as u16;
            particles.insert(id, HashMap::new());

            for (i, particle) in particle_type.iter().enumerate() {
                let particle = ParticleToSave {
                    position: particle.position,
                    velocity: particle.velocity,
                    id: particle.id,
                };
                particles.get_mut(&id).unwrap().insert(i, particle);
            }
        }
        Self {
            particles,
            particles_database,
            origin: structure.origin.into(),
        }
    }

    fn into(&self) -> Structure {
        let particle_type_count = self.particles.len();
        let mut particles = vec![];
        for particle_type in 0..particle_type_count {
            particles.push(vec![]);
            let particle_type = particle_type as u16;
            let particles_with_type = self.particles.get(&particle_type)
                .expect("No particle type");
            let particles_count = particles_with_type.len();
            for _ in 0..particles_count {
                particles[particle_type as usize].push(Particle::default());
            }
        }
        for (id, particle_type) in self.particles.iter() {
            for (i, particle) in particle_type {
                let particle: Particle = particle.into().expect("No particle in database");
                particles[*id as usize][*i] = particle;
            }
        }
        Structure {
            particles,
            origin: self.origin.into(),
        }
    }

    pub fn save_to_file (&self, path: &Path) {
        let file = if !path.exists() {
            File::create(path)
        } else {
            OpenOptions::new().truncate(true).write(true).open(path)
        }.expect("Can't write to file");
        let mut buf_writer = BufWriter::with_capacity(1073741824, file);
        serde_json::ser::to_writer(&mut buf_writer, &self)
            .expect("Can't save data to file");
    }

    pub fn load_from_file (path: &Path) -> Self {
        let file = File::open(path).expect("Can't open file to read");
        let buf_reader = BufReader::with_capacity(1073741824, file);
        let data_file: Self = serde_json::de::from_reader(buf_reader).expect("Can't read file");
        data_file
    }
}
