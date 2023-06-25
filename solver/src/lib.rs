extern crate rand_distr;
extern crate moldyn_core;
extern crate nalgebra as na;
extern crate rayon;
pub mod initializer;
pub mod solver;


#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use moldyn_core::{Particle, ParticleDatabase, State};
    use crate::initializer::InitError;
    use crate::solver::{LennardJonesPotential, Potential, update_force};
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

}
