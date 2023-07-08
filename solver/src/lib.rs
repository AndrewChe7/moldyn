extern crate moldyn_core;
extern crate nalgebra as na;
extern crate rand_distr;
extern crate rayon;
pub mod initializer;
pub mod macro_parameters;
pub mod solver;
#[macro_use]
extern crate log;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::initializer::InitError;
    use crate::macro_parameters::{get_center_of_mass_velocity, get_kinetic_energy, get_potential_energy, get_pressure, get_temperature, get_thermal_energy};
    use crate::solver::{update_force, Integrator, Potential};
    use moldyn_core::{Particle, ParticleDatabase, State};
    use na::Vector3;
    use std::sync::Mutex;

    #[test]
    fn initialize_uniform_grid() {
        let mut state = initializer::initialize_particles(8, &Vector3::new(4.0, 4.0, 4.0));
        let res = initializer::initialize_particles_position(
            &mut state,
            0,
            0,
            (0.0, 0.0, 0.0),
            (2, 2, 2),
            2.0,
        );
        assert_eq!(res, Err(InitError::ParticleIdDidNotFound));
        ParticleDatabase::add(0, "test_particle", 1.0, 0.1);
        let res = initializer::initialize_particles_position(
            &mut state,
            0,
            0,
            (0.0, 0.0, 0.0),
            (3, 3, 3),
            2.0,
        );
        assert_eq!(res, Err(InitError::TooBig));
        let res = initializer::initialize_particles_position(
            &mut state,
            0,
            0,
            (0.0, 0.0, 0.0),
            (2, 2, 2),
            2.0,
        );
        assert_eq!(res, Ok(()));
        let particle = state.particles[2].lock().expect("Can't lock particle");
        assert_eq!(particle.position.x, 0.0);
        assert_eq!(particle.position.y, 2.0);
        assert_eq!(particle.position.z, 0.0);
    }

    #[test]
    fn lennard_jones() {
        let lennard_jones_potential = Potential::new_lennard_jones(0.3418, 1.712);
        let (potential, force) = lennard_jones_potential.get_potential_and_force(0.5);
        assert_eq!(
            format!("{:.8}", potential),
            "-0.59958655"
        );
        assert_eq!(
            format!("{:.8}", force),
            "6.67445797"
        );
    }

    #[test]
    fn update_force_lennard_jones() {
        let p1 = Particle::default();
        let mut p2 = Particle::default();
        p2.position.x = 0.5;
        let mut state = State {
            particles: vec![Mutex::new(p1), Mutex::new(p2)],
            boundary_box: Vector3::new(2.0, 2.0, 2.0),
        };
        update_force(&mut state);
        let force_p1 = &state.particles[0]
            .lock()
            .expect("Can't lock particle")
            .force;
        assert_eq!(format!("{:.8}", force_p1.x), "6.67445797");
        assert_eq!(format!("{:.8}", force_p1.y), "0.00000000");
        assert_eq!(format!("{:.8}", force_p1.z), "0.00000000");
    }

    #[test]
    fn verlet_with_lennard_jones() {
        let mut p1 = Particle::default();
        let mut p2 = Particle::default();
        p1.position = Vector3::new(0.75, 0.75, 0.5);
        p2.position = Vector3::new(1.25, 0.75, 0.5);
        p1.velocity = Vector3::new(1.0, 1.0, 0.0);
        p2.velocity = Vector3::new(-1.0, 1.0, 0.0);
        p1.mass = 66.335;
        p2.mass = 66.335;
        let mut state = State {
            particles: vec![Mutex::new(p1), Mutex::new(p2)],
            boundary_box: Vector3::new(2.0, 2.0, 2.0),
        };
        let verlet = Integrator::VerletMethod;
        update_force(&mut state); // Initialize forces
        {
            let p1 = state.particles[0].lock().expect("Can't lock particle");
            let p2 = state.particles[1].lock().expect("Can't lock particle");
            let pos1 = p1.position;
            let pos2 = p2.position;
            let v1 = p1.velocity;
            let v2 = p2.velocity;
            let f1 = p1.force;
            let f2 = p2.force;

            assert_eq!(format!("{:.8}", pos1.x), "0.75000000");
            assert_eq!(format!("{:.8}", pos1.y), "0.75000000");
            assert_eq!(format!("{:.8}", pos1.z), "0.50000000");

            assert_eq!(format!("{:.8}", pos2.x), "1.25000000");
            assert_eq!(format!("{:.8}", pos2.y), "0.75000000");
            assert_eq!(format!("{:.8}", pos2.z), "0.50000000");

            assert_eq!(format!("{:.8}", v1.x), "1.00000000");
            assert_eq!(format!("{:.8}", v1.y), "1.00000000");
            assert_eq!(format!("{:.8}", v1.z), "0.00000000");

            assert_eq!(format!("{:.8}", v2.x), "-1.00000000");
            assert_eq!(format!("{:.8}", v2.y), "1.00000000");
            assert_eq!(format!("{:.8}", v2.z), "0.00000000");

            assert_eq!(format!("{:.8}", f1.x), "6.67445797");
            assert_eq!(format!("{:.8}", f1.y), "0.00000000");
            assert_eq!(format!("{:.8}", f1.z), "0.00000000");

            assert_eq!(format!("{:.8}", f2.x), "-6.67445797");
            assert_eq!(format!("{:.8}", f2.y), "0.00000000");
            assert_eq!(format!("{:.8}", f2.z), "0.00000000");
        }
        verlet.calculate(&mut state, 0.002);
        {
            let p1 = state.particles[0].lock().expect("Can't lock particle");
            let p2 = state.particles[1].lock().expect("Can't lock particle");
            let pos1 = p1.position;
            let pos2 = p2.position;
            let v1 = p1.velocity;
            let v2 = p2.velocity;
            let f1 = p1.force;
            let f2 = p2.force;
            assert_eq!(format!("{:.8}", pos1.x), "0.75200020");
            assert_eq!(format!("{:.8}", pos1.y), "0.75200000");
            assert_eq!(format!("{:.8}", pos1.z), "0.50000000");

            assert_eq!(format!("{:.8}", pos2.x), "1.24799980");
            assert_eq!(format!("{:.8}", pos2.y), "0.75200000");
            assert_eq!(format!("{:.8}", pos2.z), "0.50000000");

            assert_eq!(format!("{:.8}", f1.x), "6.97111732");
            assert_eq!(format!("{:.8}", f1.y), "0.00000000");
            assert_eq!(format!("{:.8}", f1.z), "0.00000000");

            assert_eq!(format!("{:.8}", f2.x), "-6.97111732");
            assert_eq!(format!("{:.8}", f2.y), "0.00000000");
            assert_eq!(format!("{:.8}", f2.z), "0.00000000");

            assert_eq!(format!("{:.8}", v1.x), "1.00020571");
            assert_eq!(format!("{:.8}", v1.y), "1.00000000");
            assert_eq!(format!("{:.8}", v1.z), "0.00000000");

            assert_eq!(format!("{:.8}", v2.x), "-1.00020571");
            assert_eq!(format!("{:.8}", v2.y), "1.00000000");
            assert_eq!(format!("{:.8}", v2.z), "0.00000000");
        }
    }

    #[test]
    fn energies() {
        let mut p1 = Particle::default();
        let mut p2 = Particle::default();
        p1.position = Vector3::new(0.75, 0.75, 0.5);
        p2.position = Vector3::new(1.25, 0.75, 0.5);
        p1.velocity = Vector3::new(1.0, 1.0, 0.0);
        p2.velocity = Vector3::new(-1.0, 1.0, 0.0);
        p1.mass = 66.335;
        p2.mass = 66.335;
        let mut state = State {
            particles: vec![Mutex::new(p1), Mutex::new(p2)],
            boundary_box: Vector3::new(2.0, 2.0, 2.0),
        };
        update_force(&mut state);
        let mv = get_center_of_mass_velocity(&state, 0, 2);
        assert_eq!(mv, Vector3::new(0.0, 1.0, 0.0));
        let e_kinetic = get_kinetic_energy(&state, 0, 2);
        let e_thermal = get_thermal_energy(&state, 0, 2, &mv);
        let e_potential = get_potential_energy(&state, 0, 2);
        let e = e_kinetic + e_potential;
        let e_internal = e_thermal + e_potential;
        assert_eq!(
            format!("{:.8}", e_kinetic),
            "132.67000000"
        );
        assert_eq!(
            format!("{:.8}", e_thermal),
            "66.33500000"
        );
        assert_eq!(
            format!("{:.8}", e_potential),
            "-0.59958655"
        );
        assert_eq!(
            format!("{:.8}", e_internal),
            "65.73541345"
        );
        assert_eq!(
            format!("{:.8}", e),
            "132.07041345"
        );
    }

    #[test]
    fn temperature() {
        let mut p1 = Particle::default();
        let mut p2 = Particle::default();
        p1.position = Vector3::new(0.75, 0.75, 0.5);
        p2.position = Vector3::new(1.25, 0.75, 0.5);
        p1.velocity = Vector3::new(1.0, 1.0, 0.0);
        p2.velocity = Vector3::new(-1.0, 1.0, 0.0);
        p1.mass = 66.335;
        p2.mass = 66.335;
        let mut state = State {
            particles: vec![Mutex::new(p1), Mutex::new(p2)],
            boundary_box: Vector3::new(2.0, 2.0, 2.0),
        };
        update_force(&mut state);
        let mv = get_center_of_mass_velocity(&state, 0, 2);
        let e_thermal = get_thermal_energy(&state, 0, 2, &mv);
        let temperature = get_temperature(e_thermal, 2);
        assert_eq!(
            format!("{:.8}", temperature),
            "1601.54204479"
        );
    }

    #[test]
    fn pressure() {
        let mut p1 = Particle::default();
        let mut p2 = Particle::default();
        p1.position = Vector3::new(0.75, 0.75, 0.5);
        p2.position = Vector3::new(1.25, 0.75, 0.5);
        p1.velocity = Vector3::new(1.0, 1.0, 0.0);
        p2.velocity = Vector3::new(-1.0, 1.0, 0.0);
        p1.mass = 66.335;
        p2.mass = 66.335;
        let mut state = State {
            particles: vec![Mutex::new(p1), Mutex::new(p2)],
            boundary_box: Vector3::new(2.0, 2.0, 2.0),
        };
        update_force(&mut state);
        let mv = get_center_of_mass_velocity(&state, 0, 2);
        let pressure = get_pressure(&state, 0, 2, &mv);
        assert_eq!(
            format!("{:.8}", pressure),
            "5.66696787"
        );
    }
}
