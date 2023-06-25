extern crate rand_distr;
extern crate moldyn_core;
extern crate nalgebra as na;
extern crate rayon;
pub mod initializer;
pub mod solver;


#[cfg(test)]
mod tests {
    use moldyn_core::{Particle, ParticleDatabase};
    use crate::initializer::InitError;
    use crate::solver::{LennardJonesPotential, Potential};
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
        assert_eq!(state.particles[2].position.x, 0.0);
        assert_eq!(state.particles[2].position.y, 2.0);
        assert_eq!(state.particles[2].position.z, 0.0);
    }

    #[test]
    fn lennard_jones () {
        let p1 = Particle::new();
        let mut p2 = Particle::new();
        p2.position.x = 0.5;
        let lennard_jones_potential = LennardJonesPotential::new(0.3418, 1.712);
        assert_eq!(format!("{:.8}", lennard_jones_potential.get_potential(&p1, &p2)), "-0.59958655");
        assert_eq!(format!("{:.8}", lennard_jones_potential.get_force(&p1, &p2)), "-6.67445797");
    }

}
