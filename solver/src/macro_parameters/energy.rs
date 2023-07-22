use moldyn_core::{Particle, State};
use na::Vector3;

fn particle_kinetic_energy(particle: &Particle) -> f64 {
    particle.mass * particle.velocity.dot(&particle.velocity) / 2.0
}

fn particle_thermal_energy(particle: &Particle, center_of_mass_velocity: &Vector3<f64>) -> f64 {
    let velocity = particle.velocity - center_of_mass_velocity;
    particle.mass * velocity.dot(&velocity) / 2.0
}

/// Get summary kinetic energy of particles with `particle_type_id`
pub fn get_kinetic_energy(state: &State, particle_type_id: u16) -> f64 {
    let slice = &state.particles[particle_type_id as usize][..];
    slice
        .iter()
        .map(|particle| {
            let particle = particle.read().expect("Can't lock particle");
            particle_kinetic_energy(&particle)
        })
        .sum()
}

/// Get summary thermal energy of particles with `particle_type_id`
pub fn get_thermal_energy(
    state: &State,
    particle_type_id: u16,
    center_of_mass_velocity: &Vector3<f64>,
) -> f64 {
    let slice = &state.particles[particle_type_id as usize][..];
    slice
        .iter()
        .map(|particle| {
            let particle = particle.read().expect("Can't lock particle");
            particle_thermal_energy(&particle, center_of_mass_velocity)
        })
        .sum()
}

/// Get summary potential energy of particles with `particle_type_id`
pub fn get_potential_energy(state: &State, particle_type_id: u16) -> f64 {
    let slice = &state.particles[particle_type_id as usize][..];
    let res: f64 = slice
        .iter()
        .map(|particle| {
            let particle = particle.read().expect("Can't lock particle");
            particle.potential
        })
        .sum();
    res / 2.0
}
