use moldyn_core::State;
use crate::solver::Integrator;

pub struct Solver {
    pub state: State,
    integrator: Integrator,
}

impl Solver {
    pub fn new(state: State, integrator: Integrator) -> Self {
        Self {
            state,
            integrator,
        }
    }

    pub fn solve(&mut self, delta_time: f64) {
        self.integrator.calculate(&mut self.state, delta_time);
    }

    pub fn get_final_state(self) -> State {
        self.state
    }
}
