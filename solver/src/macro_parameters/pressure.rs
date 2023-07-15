use na::Vector3;
use rayon::prelude::*;
use moldyn_core::State;

pub fn get_pressure(state: &State,
                    first_particle: usize,
                    count: usize,
                    center_of_mass_velocity: &Vector3<f64>,) -> f64 {
    let slice = &state.particles[first_particle..(first_particle + count)];
    let volume = state.boundary_box.x * state.boundary_box.y * state.boundary_box.z;
    let result: f64 = (0..slice.len()).into_par_iter().map(|i| {
        let mut res = 0.0;
        let particle = slice[i].read().expect("Can't lock particle");
        let dv = particle.velocity - center_of_mass_velocity;
        res += particle.mass * dv.x * dv.x;
        res += particle.mass * dv.y * dv.y;
        res += particle.mass * dv.z * dv.z;
        res -= particle.temp;
        res
    }).sum();
    result / volume / 3.0
}
