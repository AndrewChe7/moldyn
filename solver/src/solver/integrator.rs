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
                     barostat: &mut Option<(&mut Barostat, f64)>, thermostat: &mut Option<(&mut Thermostat, f64)>) {
        match self {
            Integrator::VerletMethod => {
                if let Some((barostat, target_pressure)) = barostat.as_mut() {
                    (0..state.particles.len()).for_each(|particle_type| {
                        barostat.calculate_myu(&state, delta_time, particle_type as u16, *target_pressure);
                    });
                }
                if let Some((thermostat, target_temperature)) = thermostat.as_mut() {
                    (0..state.particles.len()).for_each(|particle_type| {
                        thermostat.calculate_lambda(&state, delta_time, particle_type as u16, *target_temperature);
                    });
                }
                state.particles.iter_mut().for_each(|particle_type| {
                    let mass = particle_type[0].mass;
                    let temp = delta_time / (2.0 * mass);
                    particle_type.iter_mut().for_each(|particle| {
                        particle.velocity = particle.velocity + particle.force * temp;
                    });
                });
                if let Some((thermostat, target_temperature)) = thermostat.as_ref() {
                    (0..state.particles.len()).for_each(|particle_type| {
                        thermostat.update(state, delta_time, particle_type as u16, *target_temperature);
                    });
                }
                state.particles.iter_mut().for_each(|particle_type| {
                    particle_type.iter_mut().for_each(|particle| {
                        particle.position += particle.velocity * delta_time;
                    });
                });
                state.apply_boundary_conditions();
                update_force(state);
                state.particles.iter_mut().for_each(|particle_type| {
                    let mass = particle_type[0].mass;
                    let temp = delta_time / (2.0 * mass);
                    particle_type.iter_mut().for_each(|particle| {
                        particle.velocity += particle.force * temp;
                    });
                });
                if let Some((barostat, target_pressure)) = barostat.as_ref() {
                    (0..state.particles.len()).for_each(|particle_type| {
                        barostat.update(state, delta_time, particle_type as u16, *target_pressure);
                    });
                }
            }
            Integrator::Custom(_) => {
                todo!()
            }
        }
    }
}