use na::Vector3;
use moldyn_core::State;

/// Get pressure of particles with `particle_type_id`
pub fn get_pressure(state: &State,
                    particle_type_id: u16,
                    center_of_mass_velocity: &Vector3<f64>,) -> f64 {
    let slice = &state.particles[particle_type_id as usize][..];
    let volume = state.boundary_box.x * state.boundary_box.y * state.boundary_box.z;
    let result: f64 = (0..slice.len()).into_iter().map(|i| {
        let mut res = 0.0;
        let particle = &slice[i];
        let dv = particle.velocity - center_of_mass_velocity;
        res += particle.mass * dv.x * dv.x;
        res += particle.mass * dv.y * dv.y;
        res += particle.mass * dv.z * dv.z;
        res -= particle.temp;
        res
    }).sum();
    result / volume / 3.0
}
