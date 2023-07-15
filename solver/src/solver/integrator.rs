use moldyn_core::State;
use crate::initializer::{Barostat, Thermostat};
use crate::solver::update_force;

pub enum Integrator {
    VerletMethod,
    Custom(String),
}

impl Integrator {
    pub fn calculate(&self, state: &mut State, delta_time: f64,
                     mut barostat: Option<(&mut Barostat, f64)>, mut thermostat: Option<(&mut Thermostat, f64)>) {
        match self {
            Integrator::VerletMethod => {
                if let Some((barostat, target_pressure)) = barostat.as_mut() {
                    for particle_type in 0..state.particles.len() {
                        barostat.calculate_myu(&state, delta_time, particle_type as u16, *target_pressure);
                    }
                }
                if let Some((thermostat, target_temperature)) = thermostat.as_mut() {
                    for particle_type in 0..state.particles.len() {
                        thermostat.calculate_lambda(&state, delta_time, particle_type as u16, *target_temperature);
                    }
                }
                state.particles.iter_mut().for_each(|particle_type| {
                    particle_type.iter_mut().for_each(|particle| {
                        let mut particle = particle.write().expect("Can't lock particle");
                        let acceleration = particle.force / particle.mass;
                        let velocity = particle.velocity;
                        particle.position += velocity * delta_time + acceleration * delta_time * delta_time / 2.0;
                        particle.velocity += acceleration * delta_time / 2.0;
                    });
                });
                state.apply_boundary_conditions();
                if let Some((barostat, target_pressure)) = barostat.as_ref() {
                    for particle_type in 0..state.particles.len() {
                        barostat.update(state, delta_time, particle_type as u16, *target_pressure);
                    }
                }
                if let Some((thermostat, target_temperature)) = thermostat.as_ref() {
                    for particle_type in 0..state.particles.len() {
                        thermostat.update(state, delta_time, particle_type as u16, *target_temperature);
                    }
                }
                update_force(state);
                state.particles.iter_mut().for_each(|particle_type| {
                    particle_type.iter_mut().for_each(|particle| {
                        let mut particle = particle.write().expect("Can't lock particle");
                        let acceleration = particle.force / particle.mass;
                        particle.velocity += acceleration * delta_time / 2.0;
                    });
                });
            }
            Integrator::Custom(_) => {
                todo!()
            }
        }
    }
}