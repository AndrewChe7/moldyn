mod particle;
mod particles_database;
extern crate nalgebra as na;
extern crate serde;
extern crate lazy_static;
extern crate rayon;

pub use particle::Particle;
pub use particle::State;
pub use particles_database::ParticleDatabase;

pub const K_B: f64 = 1.380648528;

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Mutex;
    use na::Vector3;
    use rayon::prelude::*;
    use crate::{ParticleDatabase, State, Particle};

    fn test_particle() -> Particle {
        Particle {
            position: Vector3::new(0.1, 0.2, 0.3),
            velocity: Vector3::new(0.1, 0.2, 0.3),
            force: Vector3::new(0.3, 2.2, 1.0),
            potential: 1.0,
            mass: 2.0,
            id: 3,
        }
    }

    fn check_particle_equality(p1: &Particle, p2: &Particle) {
        assert_eq!(p1.position, p2.position);
        assert_eq!(p1.velocity, p2.velocity);
        assert_eq!(p1.force, p2.force);
        assert_eq!(p1.potential, p2.potential);
        assert_eq!(p1.mass, p2.mass);
        assert_eq!(p1.id, p2.id);
    }

    #[test]
    fn particle_serialization () {
        let particle = test_particle();

        let serialized = ron::to_string(&particle).unwrap();
        let deserialized: Particle = ron::from_str(&serialized).unwrap();

        check_particle_equality(&particle, &deserialized);
    }

    #[test]
    fn state_serialization () {
        let particle = test_particle();

        let state = State {
            particles: vec![Mutex::new(test_particle()), Mutex::new(test_particle())],
        };

        let serialized = ron::to_string(&state).unwrap();
        let deserialized: State = ron::from_str(&serialized).unwrap();

        for p in &deserialized.particles {
            let ref p = p.lock().expect("Can't lock particle");
            check_particle_equality(p, &particle);
        }
    }

    #[test]
    fn particle_database () {
        ParticleDatabase::add(0, "test_particle", 0.1337);
        ParticleDatabase::add(1, "test_particle2", 0.273);
        ParticleDatabase::add(2, "test_particle3", 0.272);

        assert_eq!(ParticleDatabase::get_particle_mass(0).unwrap(), 0.1337);
        assert_eq!(ParticleDatabase::get_particle_mass(1).unwrap(), 0.273);
        assert_eq!(ParticleDatabase::get_particle_mass(2).unwrap(), 0.272);
        assert_eq!(ParticleDatabase::get_particle_mass(3), None);
        ParticleDatabase::clear_particles();
        assert_eq!(ParticleDatabase::get_particle_mass(0), None);
        assert_eq!(ParticleDatabase::get_particle_mass(1), None);
        assert_eq!(ParticleDatabase::get_particle_mass(2), None);
        assert_eq!(ParticleDatabase::get_particle_mass(3), None);
    }

    #[test]
    fn particle_database_multithreaded () {
        let _ = (0..4).into_par_iter().for_each(|i| {
            ParticleDatabase::add(i, "test_particle", 0.1337);
        });
        assert_eq!(ParticleDatabase::get_particle_mass(0).unwrap(), 0.1337);
        assert_eq!(ParticleDatabase::get_particle_mass(1).unwrap(), 0.1337);
        assert_eq!(ParticleDatabase::get_particle_mass(2).unwrap(), 0.1337);
        assert_eq!(ParticleDatabase::get_particle_mass(3).unwrap(), 0.1337);
        let _ = (0..4).into_par_iter().for_each(|i| {
            assert_eq!(ParticleDatabase::get_particle_mass(i).unwrap(), 0.1337);
        });
    }

    use tempdir::TempDir;

    #[test]
    fn save_load_particle_database_from_file () {
        ParticleDatabase::add(0, "test_particle", 0.1);
        assert_eq!(ParticleDatabase::get_particle_mass(0).unwrap(), 0.1);
        let dir = TempDir::new("test_data")
            .expect("Can't create temp directory");
        let file_path = dir.path().join("test.ron");
        ParticleDatabase::save_particles_data(Path::new(&file_path))
            .expect("Something went wrong");
        ParticleDatabase::clear_particles();
        assert_eq!(ParticleDatabase::get_particle_mass(0), None);
        ParticleDatabase::load_particles_data(Path::new(&file_path))
            .expect("Something went wrong");
        assert_eq!(ParticleDatabase::get_particle_mass(0).unwrap(), 0.1);
    }
}
