use crate::solver::{update_force, Integrator, Potential};
use moldyn_core::State;
use rayon::prelude::*;

pub struct VerletMethod;

impl Integrator for VerletMethod {
    fn calculate(&self, state: &mut State, delta_time: f64, potential: &impl Potential) {
        state.particles.par_iter_mut().for_each(|particle| {
            let particle = particle.get_mut().expect("Can't lock particle");
            let acceleration = particle.force / particle.mass;
            particle.position +=
                particle.velocity * delta_time + acceleration * delta_time * delta_time / 2.0;
            particle.velocity += acceleration * delta_time / 2.0;
        });
        update_force(state, potential);
        state.particles.par_iter_mut().for_each(|particle| {
            let particle = particle.get_mut().expect("Can't lock particle");
            let acceleration = particle.force / particle.mass;
            particle.velocity += acceleration * delta_time / 2.0;
        });
        state.apply_boundary_conditions();
    }
}
