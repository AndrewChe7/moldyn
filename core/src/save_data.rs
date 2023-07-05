use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::path::Path;
use std::sync::Mutex;
use na::Vector3;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize, Serializer};
use crate::{Particle, ParticleDatabase, State};
use crate::particles_database::ParticleData;

#[derive(Serialize, Deserialize)]
pub struct ParticleToSave {
    /// position of particle in 3d space
    pub position: Vector3<f64>,
    /// velocity of particle
    pub velocity: Vector3<f64>,
    /// ID of particle. Defines type of particle
    pub id: u16,
}

#[derive(Serialize, Deserialize)]
pub struct StateToSave {
    #[serde(serialize_with = "ordered_map")]
    pub particles: HashMap<usize, ParticleToSave>,
    pub boundary_box: Vector3<f64>,
}

#[derive(Serialize, Deserialize)]
pub struct DataFile {
    pub state: StateToSave,
    #[serde(serialize_with = "ordered_map")]
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

impl StateToSave {
    fn from(state: &State) -> Self {
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
}

pub fn save_data_to_file (state: &State, path: &Path) {
    let particles_database = {
        let particle_database = ParticleDatabase::get_data();
        let hash_table = particle_database.lock()
            .expect("Can't lock particles database");
        hash_table.clone()
    };
    let state = StateToSave::from(state);
    let data = DataFile {
        state,
        particles_database
    };
    let file = if !path.exists() {
        File::create(path)
    } else {
        File::open(path)
    };
    let file = file.expect("Can't write to file");
    ron::ser::to_writer_pretty(file, &data, PrettyConfig::default())
        .expect("Can't save data to file");
}

pub fn load_data_from_file (path: &Path) -> State {
    let mut state = State {
        particles: vec![],
        boundary_box: Default::default(),
    };
    let file = File::open(path).expect("Can't open file to read");
    let data_file: DataFile = ron::de::from_reader(&file).expect("Can't read file");
    ParticleDatabase::load(&data_file.particles_database);
    state.boundary_box = state.boundary_box;
    state.particles = vec![];
    for (_, particle) in &data_file.state.particles {
        let particle = Particle::new(particle.id,
                                     particle.position, particle.velocity)
            .expect("Can't add particle");
        state.particles.push(Mutex::new(particle));
    }
    state
}
