extern crate rand_distr;
extern crate moldyn_core;
extern crate nalgebra as na;
extern crate rayon;
pub mod initializer;
pub mod solver;


#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use na::Vector3;
    use moldyn_core::{Particle, ParticleDatabase, State};
    use crate::initializer::InitError;
    use crate::solver::{Integrator, LennardJonesPotential, Potential, update_force, VerletMethod};
    use super::*;

    #[test]
    fn initialize_uniform_grid () {
        let mut state = initializer::initialize_particles(8);
        let res = initializer::initialize_particles_position(
            &mut state,
            0,
            0,
            (0.0, 0.0, 0.0),
            (2, 2, 2),
            2.0);
        assert_eq!(res, Err(InitError::ParticleIdDidNotFound));
        ParticleDatabase::add(0, "test_particle", 1.0);
        let res = initializer::initialize_particles_position(
            &mut state,
            0,
            0,
            (0.0, 0.0, 0.0),
            (3, 3, 3),
            2.0);
        assert_eq!(res, Err(InitError::TooBig));
        let res = initializer::initialize_particles_position(
            &mut state,
            0,
            0,
            (0.0, 0.0, 0.0),
            (2, 2, 2),
            2.0);
        assert_eq!(res, Ok(()));
        let particle = state.particles[2].lock().expect("Can't lock particle");
        assert_eq!(particle.position.x, 0.0);
        assert_eq!(particle.position.y, 2.0);
        assert_eq!(particle.position.z, 0.0);
    }

    #[test]
    fn lennard_jones () {
        let lennard_jones_potential = LennardJonesPotential::new(0.3418, 1.712);
        assert_eq!(format!("{:.8}", lennard_jones_potential.get_potential(0.5)), "-0.59958655");
        assert_eq!(format!("{:.8}", lennard_jones_potential.get_force(0.5)), "6.67445797");
    }

    #[test]
    fn update_force_lennard_jones () {
        let p1 = Particle::new();
        let mut p2 = Particle::new();
        p2.position.x = 0.5;
        let mut state = State {
            particles: vec![Mutex::new(p1), Mutex::new(p2)],
        };
        let lennard_jones_potential = LennardJonesPotential::new(0.3418, 1.712);
        update_force(&mut state, &lennard_jones_potential);
        let force_p1 = &state.particles[0].lock().expect("Can't lock particle").force;
        assert_eq!(format!("{:.8}", force_p1.x), "6.67445797");
        assert_eq!(format!("{:.8}", force_p1.y), "0.00000000");
        assert_eq!(format!("{:.8}", force_p1.z), "0.00000000");
    }

    #[test]
    fn verlet_with_lennard_jones () {
        let mut p1 = Particle::new();
        let mut p2 = Particle::new();
        p1.position = Vector3::new(0.75, 0.75, 0.5);
        p2.position = Vector3::new(1.25, 0.75, 0.5);
        p1.velocity = Vector3::new(1.0, 1.0, 0.0);
        p2.velocity = Vector3::new(-1.0, 1.0, 0.0);
        p1.mass = 66.335;
        p2.mass = 66.335;
        let mut state = State {
            particles: vec![Mutex::new(p1), Mutex::new(p2)],
        };
        let lennard_jones_potential = LennardJonesPotential::new(0.3418, 1.712);
        let verlet = VerletMethod;
        update_force(&mut state, &lennard_jones_potential); // Initialize forces
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
        { // Step
            verlet.calculate_before_force(&mut state, 0.002);
            update_force(&mut state, &lennard_jones_potential);
            verlet.calculate_after_force(&mut state, 0.002);
        }
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

}
