mod energy;
mod temperature;

pub use energy::*;
use moldyn_core::State;
use na::{Vector3, Vector4};
use rayon::prelude::*;
pub use temperature::*;

/// Get velocity of center of mass of particles [first_particle]..[first_particle]+[count]
pub fn get_center_of_mass_velocity(
    state: &State,
    first_particle: usize,
    count: usize,
) -> Vector3<f64> {
    let slice = &state.particles[first_particle..(first_particle + count)];
    let res: Vector4<f64> = slice
        .into_par_iter()
        .map(|particle| {
            let particle = particle.lock().expect("Can't lock particle");
            let v = particle.velocity;
            Vector4::new(v.x, v.y, v.z, 1.0) * particle.mass
        })
        .sum();
    Vector3::new(res.x, res.y, res.z) / res.w
}
