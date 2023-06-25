use rayon::prelude::*;
use moldyn_core::State;
use crate::solver::{Integrator, Potential, update_force};

pub struct VerletMethod;

impl Integrator for VerletMethod {
    fn calculate(&self, state: &mut State, delta_time: f64, potential: &impl Potential) {
        state.particles.par_iter_mut().for_each(|particle| {
            let particle = particle.get_mut().expect("Can't lock particle");
            let acceleration = particle.force / particle.mass;
            particle.position += particle.velocity * delta_time +
                acceleration * delta_time * delta_time / 2.0;
            particle.velocity += acceleration * delta_time / 2.0;
        });
        update_force(state, potential);
        state.particles.par_iter_mut().for_each(|particle| {
            let particle = particle.get_mut().expect("Can't lock particle");
            let acceleration = particle.force / particle.mass;
            particle.velocity += acceleration * delta_time / 2.0;
        });
    }
}