use na::Vector3;
use rayon::prelude::*;
use moldyn_core::{Particle, State};

fn particle_kinetic_energy(particle: &Particle) -> f64 {
    particle.mass * particle.velocity.dot(&particle.velocity) / 2.0
}

fn particle_thermal_energy(particle: &Particle, center_of_mass_velocity: &Vector3<f64>) -> f64 {
    let velocity = particle.velocity - center_of_mass_velocity;
    particle.mass * velocity.dot(&velocity) / 2.0
}

pub fn get_kinetic_energy(state: &State, first_particle: usize, count: usize) -> f64 {
    let slice = &state.particles[first_particle..(first_particle + count)];
    slice.into_par_iter().map(|particle| {
        let particle = particle.lock()
            .expect("Can't lock particle");
        particle_kinetic_energy(&particle)
    }).sum()
}

pub fn get_thermal_energy(state: &State, first_particle: usize, count: usize, center_of_mass_velocity: &Vector3<f64>) -> f64 {
    let slice = &state.particles[first_particle..(first_particle + count)];
    slice.into_par_iter().map(|particle| {
        let particle = particle.lock()
            .expect("Can't lock particle");
        particle_thermal_energy(&particle, center_of_mass_velocity)
    }).sum()
}