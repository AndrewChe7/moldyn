use moldyn_core::State;

pub trait Integrator {
    fn calculate_before_force(state: &mut State);
    fn calculate_after_force(state: &mut State);
}