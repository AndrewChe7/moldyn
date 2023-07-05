use moldyn_core::State;
use rayon::prelude::*;
use crate::solver::update_force;

pub enum Integrator {
    VerletMethod,
    Custom(String),
}

impl Integrator {
    pub fn calculate(&self, state: &mut State, delta_time: f64) {
        match self {
            Integrator::VerletMethod => {
                state.particles.par_iter_mut().for_each(|particle| {
                    let particle = particle.get_mut().expect("Can't lock particle");
                    let acceleration = particle.force / particle.mass;
                    particle.position +=
                        particle.velocity * delta_time + acceleration * delta_time * delta_time / 2.0;
                    particle.velocity += acceleration * delta_time / 2.0;
                });
                update_force(state);
                state.particles.par_iter_mut().for_each(|particle| {
                    let particle = particle.get_mut().expect("Can't lock particle");
                    let acceleration = particle.force / particle.mass;
                    particle.velocity += acceleration * delta_time / 2.0;
                });
                state.apply_boundary_conditions();
            }
            Integrator::Custom(_) => {
                todo!()
            }
        }
    }
}