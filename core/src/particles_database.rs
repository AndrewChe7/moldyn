use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs::File;
use std::option::Option;
use std::path::Path;
use std::sync::Mutex;
use std::string::String;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ParticleData {
    name: String,
    mass: f64,
}

lazy_static!(
    static ref PARTICLE_DATA: Mutex<HashMap<u16, ParticleData>> = Mutex::new(HashMap::new());
);

#[derive(Debug)]
pub enum SaveLoadError {
    CantOpen,
    CantCreate,
    CantWrite,
    CantRead,
}

pub struct ParticleDatabase;

impl ParticleDatabase {
    pub fn add(id: u16, name: &str, mass: f64) {
        let mut particle_data_locked = PARTICLE_DATA.lock()
            .expect("Can't lock mutex");
        particle_data_locked.insert(id, ParticleData {name: String::from(name), mass});
    }

    pub fn get_particle_mass(id: u16) -> Option<f64> {
        let particle_data_locked = PARTICLE_DATA.lock()
            .expect("Can't lock mutex");
        if particle_data_locked.contains_key(&id) {
            Some(particle_data_locked.get(&id).unwrap().mass)
        } else {
            None
        }
    }

    pub fn get_particle_name(id: u16) -> Option<String> {
        let particle_data_locked = PARTICLE_DATA.lock()
            .expect("Can't lock mutex");
        if particle_data_locked.contains_key(&id) {
            Some(particle_data_locked.get(&id).unwrap().name.clone())
        } else {
            None
        }
    }

    pub fn clear_particles() {
        let mut particle_data_locked = PARTICLE_DATA.lock()
            .expect("Can't lock mutex");
        particle_data_locked.clear();
    }

    pub fn save_particles_data (path: &Path) -> Result<(), SaveLoadError> {
        let file = if !path.exists() {
            File::create(path)
        } else {
            File::open(path)
        };
        if file.is_err() {
            return Err(SaveLoadError::CantOpen);
        }
        let file = file.unwrap();
        {
            let particle_data_locked = PARTICLE_DATA.lock()
                .expect("Can't lock mutex");
            let data = &*particle_data_locked;
            let res = ron::ser::to_writer_pretty(file, data, PrettyConfig::default());
            if res.is_err() {
                return Err(SaveLoadError::CantWrite);
            }
        }
        Ok(())
    }

    pub fn load_particles_data (path: &Path) -> Result<(), SaveLoadError> {
        let file = File::open(path);
        if file.is_err() {
            return Err(SaveLoadError::CantOpen);
        }
        let file = file.unwrap();
        let res = ron::de::from_reader(file);
        if res.is_err() {
            return Err(SaveLoadError::CantRead);
        }
        let particles_data: HashMap<u16, ParticleData> = res.unwrap();
        {
            let mut particle_data_locked = PARTICLE_DATA.lock()
                .expect("Can't lock mutex");
            for particle_data in particles_data {
                if !particle_data_locked.contains_key(&particle_data.0) {
                    particle_data_locked.insert(particle_data.0, particle_data.1);
                }
            }
        }
        Ok(())
    }
}