use rayon::prelude::*;
use moldyn_core::State;
use crate::solver::integrator::Integrator;

pub struct VerletMethod;

impl Integrator for VerletMethod {
    fn calculate_before_force(&self, state: &mut State, delta_time: f64) {
        state.particles.par_iter_mut().for_each(|particle| {
            let particle = particle.get_mut().expect("Can't lock particle");
            let acceleration = particle.force / particle.mass;
            particle.position =
                particle.position +
                particle.velocity * delta_time +
                acceleration * delta_time * delta_time / 2.0;
            particle.velocity = particle.velocity + acceleration * delta_time / 2.0;
        });
    }

    fn calculate_after_force(&self, state: &mut State, delta_time: f64) {
        state.particles.par_iter_mut().for_each(|particle| {
            let particle = particle.get_mut().expect("Can't lock particle");
            let acceleration = particle.force / particle.mass;
            particle.velocity = particle.velocity + acceleration * delta_time / 2.0;
        });
    }
}