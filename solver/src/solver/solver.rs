use moldyn_core::State;
use crate::solver::{Integrator, Potential};

pub struct Solver<I, P> {
    pub state: State,
    integrator: I,
    potential: P,
}

impl<I: Integrator, P: Potential> Solver<I, P> {
    pub fn new(state: State, integrator: I, potential: P) -> Self {
        Self {
            state,
            integrator,
            potential
        }
    }

    pub fn solve(&mut self, delta_time: f64) {
        self.integrator.calculate(&mut self.state, delta_time, &self.potential);
    }

    pub fn get_final_state(self) -> State {
        self.state
    }
}
