use moldyn_core::State;
use crate::solver::Potential;

pub trait Integrator {
    fn calculate(&self, state: &mut State, delta_time: f64, potential: &impl Potential);
}