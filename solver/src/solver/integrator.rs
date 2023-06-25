use moldyn_core::State;

pub trait Integrator {
    fn calculate_before_force(&self, state: &mut State, delta_time: f64);
    fn calculate_after_force(&self, state: &mut State, delta_time: f64);
}