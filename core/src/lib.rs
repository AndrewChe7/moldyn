mod particle;
mod particles_database;
mod save_data;

extern crate lazy_static;
extern crate nalgebra as na;
extern crate serde;

pub use particle::Particle;
pub use particle::State;
pub use particles_database::ParticleDatabase;
pub use save_data::*;

pub const K_B: f64 = 1.380648528;

#[cfg(test)]
mod tests {
    use crate::{Particle, ParticleDatabase, ParticleToSave, State, StateToSave};
    use na::Vector3;
    use rand::Rng;
    use std::path::Path;
    use std::sync::RwLock;

    fn test_particle() -> Particle {
        ParticleDatabase::add(3, "test", 2.0, 0.2);
        Particle::new(3, Vector3::new(0.1, 0.2, 0.3),
                      Vector3::new(0.1, 0.2, 0.3)).unwrap()
    }

    fn check_particle_equality(p1: &Particle, p2: &Particle) {
        assert_eq!(p1.position, p2.position);
        assert_eq!(p1.velocity, p2.velocity);
        assert_eq!(p1.mass, p2.mass);
        assert_eq!(p1.id, p2.id);
    }

    #[test]
    fn particle_serialization() {
        let particle = test_particle();
        let particle_data_to_save = ParticleToSave::from(&particle);
        let serialized = serde_json::to_string(&particle_data_to_save).unwrap();
        let deserialized_data: ParticleToSave = serde_json::from_str(&serialized).unwrap();
        let converted: Particle = (&deserialized_data).into().unwrap();
        check_particle_equality(&particle, &converted);
    }

    #[test]
    fn state_serialization() {
        let particle = test_particle();
        let state = State {
            particles: vec![vec![RwLock::new(test_particle()), RwLock::new(test_particle())]],
            boundary_box: Vector3::new(2.0, 2.0, 2.0),
        };
        let state_data_to_save = StateToSave::from(&state);
        let serialized = serde_json::to_string(&state_data_to_save).unwrap();
        let deserialized: StateToSave = serde_json::from_str(&serialized).unwrap();
        let deserialized: State = (&deserialized).into();
        for p in &deserialized.particles[0] {
            let ref p = p.read().expect("Can't lock particle");
            check_particle_equality(p, &particle);
        }
    }

    #[test]
    fn particle_database() {
        ParticleDatabase::add(0, "test_particle", 0.1337, 0.01337);
        ParticleDatabase::add(1, "test_particle2", 0.273, 0.0273);
        ParticleDatabase::add(2, "test_particle3", 0.272, 0.0272);

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
    fn particle_database_multithreaded() {
        let _ = (0..4).into_iter().for_each(|i| {
            ParticleDatabase::add(i, "test_particle", 0.1337, 0.01337);
        });
        assert_eq!(ParticleDatabase::get_particle_mass(0).unwrap(), 0.1337);
        assert_eq!(ParticleDatabase::get_particle_mass(1).unwrap(), 0.1337);
        assert_eq!(ParticleDatabase::get_particle_mass(2).unwrap(), 0.1337);
        assert_eq!(ParticleDatabase::get_particle_mass(3).unwrap(), 0.1337);
        let _ = (0..4).into_iter().for_each(|i| {
            assert_eq!(ParticleDatabase::get_particle_mass(i).unwrap(), 0.1337);
        });
    }

    use tempdir::TempDir;

    #[test]
    fn save_load_particle_database_from_file() {
        ParticleDatabase::add(0, "test_particle", 0.1, 0.01);
        assert_eq!(ParticleDatabase::get_particle_mass(0).unwrap(), 0.1);
        let dir = TempDir::new("test_data").expect("Can't create temp directory");
        let file_path = dir.path().join("test.ron");
        ParticleDatabase::save_particles_data(Path::new(&file_path)).expect("Something went wrong");
        ParticleDatabase::clear_particles();
        assert_eq!(ParticleDatabase::get_particle_mass(0), None);
        ParticleDatabase::load_particles_data(Path::new(&file_path)).expect("Something went wrong");
        assert_eq!(ParticleDatabase::get_particle_mass(0).unwrap(), 0.1);
    }

    fn check_boundary_conditions(state: &State) -> bool {
        let bb = &state.boundary_box;
        let slice = state.particles[0].as_slice();
        slice.into_iter().all(|particle| {
            let particle = particle.read().expect("Can't lock particle");
            particle.position.x >= 0.0
                && particle.position.x <= bb.x
                && particle.position.y >= 0.0
                && particle.position.y <= bb.y
                && particle.position.z >= 0.0
                && particle.position.z <= bb.z
        })
    }

    #[test]
    fn boundary_conditions_test() {
        let mut p = Particle::default();
        let mut rng = rand::thread_rng();
        p.position.x = rng.gen();
        p.position.y = rng.gen();
        p.position.z = 3.0;
        let mut state = State {
            particles: vec![vec![RwLock::new(p)]],
            boundary_box: Vector3::new(1.0, 1.0, 1.0),
        };
        assert!(!check_boundary_conditions(&state));
        state.apply_boundary_conditions();
        assert!(check_boundary_conditions(&state));
    }
}
