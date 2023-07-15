use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::option::Option;
use std::path::Path;
use std::string::String;
use std::sync::RwLock;

#[derive(Serialize, Deserialize, Clone)]
pub struct ParticleData {
    pub name: String,
    pub mass: f64,
    pub radius: f64,
}

lazy_static! {
    static ref PARTICLE_DATA: RwLock<HashMap<u16, ParticleData>> = RwLock::new(HashMap::new());
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum SaveLoadError {
    CantOpen,
    CantCreate,
    CantWrite,
    CantRead,
}

pub struct ParticleDatabase;

impl ParticleDatabase {
    pub fn add(id: u16, name: &str, mass: f64, radius: f64) {
        let mut particle_data_locked = PARTICLE_DATA.write().expect("Can't lock mutex");
        particle_data_locked.insert(
            id,
            ParticleData {
                name: String::from(name),
                mass,
                radius,
            },
        );
    }

    pub fn get_particle_mass(id: u16) -> Option<f64> {
        let particle_data_locked = PARTICLE_DATA.read().expect("Can't lock mutex");
        if particle_data_locked.contains_key(&id) {
            Some(particle_data_locked.get(&id).unwrap().mass)
        } else {
            None
        }
    }

    pub fn get_data() -> &'static RwLock<HashMap<u16, ParticleData>> {
        &PARTICLE_DATA
    }

    pub fn get_particle_radius(id: u16) -> Option<f64> {
        let particle_data_locked = PARTICLE_DATA.read().expect("Can't lock mutex");
        if particle_data_locked.contains_key(&id) {
            Some(particle_data_locked.get(&id).unwrap().radius)
        } else {
            None
        }
    }

    pub fn get_particle_name(id: u16) -> Option<String> {
        let particle_data_locked = PARTICLE_DATA.read().expect("Can't lock mutex");
        if particle_data_locked.contains_key(&id) {
            Some(particle_data_locked.get(&id).unwrap().name.clone())
        } else {
            None
        }
    }

    pub fn clear_particles() {
        let mut particle_data_locked = PARTICLE_DATA.write().expect("Can't lock mutex");
        particle_data_locked.clear();
    }

    pub fn save_particles_data(path: &Path) -> Result<(), SaveLoadError> {
        let file = if !path.exists() {
            File::create(path)
        } else {
            OpenOptions::new().truncate(true).write(true).open(path)
        };
        if file.is_err() {
            return Err(SaveLoadError::CantOpen);
        }
        let file = file.unwrap();
        let mut buf_writer = BufWriter::new(file);
        {
            let particle_data_locked = PARTICLE_DATA.read().expect("Can't lock mutex");
            let data = &*particle_data_locked;
            let res = serde_json::ser::to_writer_pretty(&mut buf_writer, data);
            if res.is_err() {
                return Err(SaveLoadError::CantWrite);
            }
        }
        Ok(())
    }

    pub fn load_particles_data(path: &Path) -> Result<(), SaveLoadError> {
        let file = File::open(path);
        if file.is_err() {
            return Err(SaveLoadError::CantOpen);
        }
        let file = file.unwrap();
        let buf_reader = BufReader::new(file);
        let res = serde_json::de::from_reader(buf_reader);
        if res.is_err() {
            return Err(SaveLoadError::CantRead);
        }
        let particles_data: HashMap<u16, ParticleData> = res.unwrap();
        {
            let mut particle_data_locked = PARTICLE_DATA.write().expect("Can't lock mutex");
            for particle_data in particles_data {
                particle_data_locked
                    .entry(particle_data.0)
                    .or_insert(particle_data.1);
            }
        }
        Ok(())
    }

    pub fn load(particle_database: &HashMap<u16, ParticleData>) {
        for (id, particle) in particle_database {
            ParticleDatabase::add(id.clone(), particle.name.as_str(),
                                  particle.mass.clone(), particle.radius.clone());
        }
    }
}
