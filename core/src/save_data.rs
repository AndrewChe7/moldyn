use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::Mutex;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use crate::{ParticleDatabase, State};
use crate::particles_database::ParticleData;

#[derive(Serialize, Deserialize)]
pub struct DataFile {
    pub state: State,
    pub particles_database: HashMap<u16, ParticleData>,
}

pub fn save_data_to_file (state: &State, path: &Path) {
    let mut particles_database = HashMap::new();
    {
        let particle_database = ParticleDatabase::get_data();
        let hash_table = particle_database.lock()
            .expect("Can't lock particles database");
        particles_database = hash_table.clone();
    }
    let state = state.clone();
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

pub fn load_data_from_file (state: &mut State, path: &Path) {
    let file = File::open(path).expect("Can't open file to read");
    let data_file: DataFile = ron::de::from_reader(&file).expect("Can't read file");
    ParticleDatabase::load(&data_file.particles_database);
    state.boundary_box = state.boundary_box;
    state.particles = vec![];
    for particle in &data_file.state.particles {
        let particle = particle.lock().expect("Can't lock particle");
        state.particles.push(Mutex::new(particle.clone()));
    }
}