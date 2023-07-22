use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::option::Option;
use std::path::Path;
use std::string::String;
use std::sync::RwLock;

/// It keeps particle type data in `ParticleDatabase`.
#[derive(Serialize, Deserialize, Clone)]
pub struct ParticleData {
    /// Name of a particle
    pub name: String,
    /// Particle mass in 10^(-27) kg
    pub mass: f64,
    /// Particle radius in nm
    pub radius: f64,
}

lazy_static! {
    static ref PARTICLE_DATA: RwLock<HashMap<u16, ParticleData>> = RwLock::new(HashMap::new());
}

/// Custom read/write file errors
/// * Can't open
/// * Can't create
/// * Can't write
/// * Can't read
#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum SaveLoadError {
    CantOpen,
    CantCreate,
    CantWrite,
    CantRead,
}

/// Empty structure that allows access to particle database from static variable
pub struct ParticleDatabase;

impl ParticleDatabase {
    /// Add particle to database.
    ///
    /// # Arguments
    ///
    /// * `id` - ID of particle in particle database.
    /// * `name` - particle name
    /// * `mass` - particle mass in 10^(-27) kg
    /// * `radius` - particle radius in nm
    ///
    /// # Panics
    ///
    /// This function can panic if it can't lock particle database.
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

    /// Gets mass of particle with `id`
    ///
    /// # Returns
    ///
    /// Particle mass if it exists in particle database else it returns None
    ///
    /// # Panics
    ///
    /// This function can panic if it can't lock particle database.
    pub fn get_particle_mass(id: u16) -> Option<f64> {
        let particle_data_locked = PARTICLE_DATA.read().expect("Can't lock mutex");
        if particle_data_locked.contains_key(&id) {
            Some(particle_data_locked.get(&id).unwrap().mass)
        } else {
            None
        }
    }

    /// Returns reference to static object
    pub fn get_data() -> &'static RwLock<HashMap<u16, ParticleData>> {
        &PARTICLE_DATA
    }

    /// Gets radius of particle with `id`
    ///
    /// # Returns
    ///
    /// Particle radius if it exists in particle database else it returns None
    ///
    /// # Panics
    ///
    /// This function can panic if it can't lock particle database.
    pub fn get_particle_radius(id: u16) -> Option<f64> {
        let particle_data_locked = PARTICLE_DATA.read().expect("Can't lock mutex");
        if particle_data_locked.contains_key(&id) {
            Some(particle_data_locked.get(&id).unwrap().radius)
        } else {
            None
        }
    }

    /// Gets name of particle with `id`
    ///
    /// # Returns
    ///
    /// Particle name if it exists in particle database else it returns None
    ///
    /// # Panics
    ///
    /// This function can panic if it can't lock particle database.
    pub fn get_particle_name(id: u16) -> Option<String> {
        let particle_data_locked = PARTICLE_DATA.read().expect("Can't lock mutex");
        if particle_data_locked.contains_key(&id) {
            Some(particle_data_locked.get(&id).unwrap().name.clone())
        } else {
            None
        }
    }

    /// Remove all particles from database.
    ///
    /// # Panics
    ///
    /// This function can panic if it can't lock particle database.
    pub fn clear_particles() {
        let mut particle_data_locked = PARTICLE_DATA.write().expect("Can't lock mutex");
        particle_data_locked.clear();
    }

    /// Serializes particle database to file.
    ///
    /// # Returns
    ///
    /// Ok(()) if there is no error else returns [SaveLoadError]
    ///
    /// # Panics
    ///
    /// This function can panic if it can't lock particle database.
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

    /// Load particle database from file
    ///
    /// # Returns
    ///
    /// Ok(()) if there is no error else returns [SaveLoadError]
    ///
    /// # Panics
    ///
    /// This function can panic if it can't lock particle database.
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

    /// Loads particle database from loaded database
    pub fn load(particle_database: &HashMap<u16, ParticleData>) {
        for (id, particle) in particle_database {
            ParticleDatabase::add(*id, particle.name.as_str(),
                                  particle.mass, particle.radius);
        }
    }
}
