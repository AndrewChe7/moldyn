use lazy_static::lazy_static;
use std::collections::HashMap;
use std::option::Option;
use std::sync::Mutex;
use std::string::String;

struct ParticleData {
    name: String,
    mass: f64,
}

lazy_static!(
    static ref PARTICLE_DATA: Mutex<HashMap<u16, ParticleData>> = Mutex::new(HashMap::new());
);

pub struct ParticleDatabase;

impl ParticleDatabase {
    pub fn add(id: u16, name: &str, mass: f64) {
        let mut particle_data_locked = PARTICLE_DATA.lock().expect("Can't lock mutex");
        particle_data_locked.insert(id, ParticleData {name: String::from(name), mass});
    }

    pub fn get_particle_mass(id: u16) -> Option<f64> {
        let particle_data_locked = PARTICLE_DATA.lock().expect("Can't lock mutex");
        if particle_data_locked.contains_key(&id) {
            Some(particle_data_locked.get(&id).unwrap().mass)
        } else {
            None
        }
    }

    pub fn get_particle_name(id: u16) -> Option<String> {
        let particle_data_locked = PARTICLE_DATA.lock().expect("Can't lock mutex");
        if particle_data_locked.contains_key(&id) {
            Some(particle_data_locked.get(&id).unwrap().name.clone())
        } else {
            None
        }
    }

    pub fn clear_particles() {
        let mut particle_data_locked = PARTICLE_DATA.lock().expect("Can't lock mutex");
        particle_data_locked.clear();
    }
}