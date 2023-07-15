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
                    barostat.calculate_myu(&state, delta_time, *target_pressure);
                }
                if let Some((thermostat, target_temperature)) = thermostat.as_mut() {
                    thermostat.calculate_lambda(&state, delta_time, *target_temperature);
                }
                state.particles.iter_mut().for_each(|particle| {
                    let particle = particle.get_mut().expect("Can't lock particle");
                    let acceleration = particle.force / particle.mass;
                    particle.position +=
                        particle.velocity * delta_time + acceleration * delta_time * delta_time / 2.0;
                    particle.velocity += acceleration * delta_time / 2.0;
                });
                state.apply_boundary_conditions();
                if let Some((barostat, target_pressure)) = barostat.as_ref() {
                    barostat.update(state, delta_time, *target_pressure);
                }
                if let Some((thermostat, target_temperature)) = thermostat.as_ref() {
                    thermostat.update(state, delta_time, *target_temperature);
                }
                update_force(state);
                state.particles.iter_mut().for_each(|particle| {
                    let particle = particle.get_mut().expect("Can't lock particle");
                    let acceleration = particle.force / particle.mass;
                    particle.velocity += acceleration * delta_time / 2.0;
                });
            }
            Integrator::Custom(_) => {
                todo!()
            }
        }
    }
}