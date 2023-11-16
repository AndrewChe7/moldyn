use moldyn_core::State;
use crate::initializer::{Barostat, Thermostat};
use crate::solver::update_force;

pub enum Integrator {
    /// <https://doi.org/10.1103/PhysRev.159.98>
    VerletMethod,
    /// Doesn't implemented now
    Custom(String),
}

impl Integrator {
    /// Just integrator iteration
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
                        let acceleration = particle.force / particle.mass;
                        particle.velocity = particle.velocity + acceleration * delta_time / 2.0;
                    });
                });
                if let Some((thermostat, target_temperature)) = thermostat.as_ref() {
                    for particle_type in 0..state.particles.len() {
                        thermostat.update(state, delta_time, particle_type as u16, *target_temperature);
                    }
                }
                state.particles.iter_mut().for_each(|particle_type| {
                    particle_type.iter_mut().for_each(|particle| {
                        let velocity = particle.velocity;
                        particle.position += velocity * delta_time;
                    });
                });
                state.apply_boundary_conditions();
                update_force(state);
                state.particles.iter_mut().for_each(|particle_type| {
                    particle_type.iter_mut().for_each(|particle| {
                        let acceleration = particle.force / particle.mass;
                        particle.velocity += acceleration * delta_time / 2.0;
                    });
                });
                if let Some((barostat, target_pressure)) = barostat.as_ref() {
                    for particle_type in 0..state.particles.len() {
                        barostat.update(state, delta_time, particle_type as u16, *target_pressure);
                    }
                }
            }
            Integrator::Custom(_) => {
                todo!()
            }
        }
    }
}