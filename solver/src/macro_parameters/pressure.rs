use na::Vector3;
use moldyn_core::State;

/// Get pressure of particles with `particle_type_id`
pub fn get_pressure(state: &State,
                    particle_type_id: u16,
                    center_of_mass_velocity: &Vector3<f64>,) -> f64 {
    let slice = &state.particles[particle_type_id as usize][..];
    let volume = state.boundary_box.x * state.boundary_box.y * state.boundary_box.z;
    let mut result1 = 0.0;
    let mut result2 = 0.0;
    for particle in slice {
        let dv = particle.velocity - center_of_mass_velocity;
        result1 += particle.mass * dv.x * dv.x;
        result1 += particle.mass * dv.y * dv.y;
        result1 += particle.mass * dv.z * dv.z;
        result2 -= particle.temp;
    }
    (result1 + result2 * 0.5) / volume / 3.0
}
