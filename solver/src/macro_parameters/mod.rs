mod energy;
mod temperature;
mod pressure;

pub use energy::*;
use moldyn_core::State;
use na::{Vector3, Vector4};
pub use temperature::*;
pub use pressure::*;

/// Get velocity of center of mass of particles with id [particle_type_id]
pub fn get_center_of_mass_velocity(
    state: &State,
    particle_type_id: u16,
) -> Vector3<f64> {
    let slice = &state.particles[particle_type_id as usize][..];
    let res: Vector4<f64> = slice
        .into_iter()
        .map(|particle| {
            let particle = particle.read().expect("Can't lock particle");
            let v = particle.velocity;
            Vector4::new(v.x, v.y, v.z, 1.0) * particle.mass
        })
        .sum();
    Vector3::new(res.x, res.y, res.z) / res.w
}
