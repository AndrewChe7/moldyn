use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::path::Path;
use std::sync::Mutex;
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

#[derive(Serialize, Deserialize, Clone)]
pub struct DataFile {
    #[serde(serialize_with = "ordered_map")]
    pub frames: HashMap<usize, StateToSave>,
    #[serde(serialize_with = "ordered_map")]
    pub particles_database: HashMap<u16, ParticleData>,
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
            let particle = particle.lock().expect("Can't lock particle");
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
            particles.push(Mutex::new(Particle::default()));
        }
        let boundary_box = self.boundary_box.clone();
        for (i, particle) in self.particles.iter() {
            let particle: Particle = particle.into().expect("No particle in database");
            particles[i.clone()] = Mutex::new(particle);
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
            let hash_table = particle_database.lock()
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

    pub fn add_state(&mut self, state: &State) {
        let state = StateToSave::from(state);
        let last_frame = self.start_frame + self.frame_count;
        self.frames.insert(last_frame, state);
        self.frame_count += 1;
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
        let result = DataFile {
            frames,
            particles_database,
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
        for (i, frame) in &data.frames {
            let t = *i - data.start_frame;
            if t < frame_count_for_first_part {
                a.insert(i.clone(), frame.clone());
            } else {
                b.insert(i.clone(), frame.clone());
            }
        }
        let left = DataFile {
            frames: a,
            particles_database: data.particles_database.clone(),
            start_frame: data.start_frame,
            frame_count: frame_count_for_first_part,
        };
        let right = DataFile {
            frames: b,
            particles_database: data.particles_database.clone(),
            start_frame: data.start_frame + frame_count_for_first_part,
            frame_count: data.frame_count - frame_count_for_first_part,
        };
        let result = (left, right);
        Ok(result)
    }

    pub fn save_to_file (&self, path: &Path) {
        let file = if !path.exists() {
            File::create(path)
        } else {
            File::open(path)
        }.expect("Can't write to file");
        serde_json::ser::to_writer_pretty(file, &self)
            .expect("Can't save data to file");
    }

    pub fn load_from_file (path: &Path) -> Self {
        let file = File::open(path).expect("Can't open file to read");
        let data_file: Self = serde_json::de::from_reader(&file).expect("Can't read file");
        data_file
    }

    pub fn get_last_frame(&self) -> State {
        let last_frame_number = self.start_frame + self.frame_count - 1;
        let last_frame = self.frames.get(&last_frame_number)
            .expect("Can't get frame");
        last_frame.into()
    }
}
