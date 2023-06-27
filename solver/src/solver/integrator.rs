use crate::solver::Potential;
use moldyn_core::State;

pub trait Integrator {
    fn calculate(&self, state: &mut State, delta_time: f64, potential: &impl Potential);
}
