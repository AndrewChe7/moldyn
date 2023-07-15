use crate::macro_parameters::{get_center_of_mass_velocity, get_temperature, get_thermal_energy};

pub enum Thermostat {
    Berendsen {
        tau: f64,
        lambda: f64,
    },
    Custom {
        name: String,
        custom_data: Vec<f64>,
    }
}

impl Thermostat {
    pub fn calculate_lambda(&mut self, state: &moldyn_core::State, delta_time: f64,
                            particle_type_id: u16, target_temperature: f64) {
        let particles_count = state.particles[particle_type_id as usize].len();
        let mv = get_center_of_mass_velocity(&state, particle_type_id);
        let thermal_energy = get_thermal_energy(&state, particle_type_id, &mv);
        let temperature = get_temperature(thermal_energy, particles_count);
        match self {
            Thermostat::Berendsen { tau, lambda } => {
                let lambda_squared = 1.0 + delta_time / *tau * (target_temperature / temperature - 1.0);
                *lambda = lambda_squared.sqrt();
            }
            Thermostat::Custom { .. } => {
                todo!()
            }
        }
    }

    pub fn update(&self, state: &mut moldyn_core::State, _delta_time: f64,
                  particle_type_id: u16, _target_temperature: f64) {
        match self {
            Thermostat::Berendsen {lambda, ..} => {
                for particle in state.particles[particle_type_id as usize].iter_mut() {
                    let mut particle = particle.write().expect("Can't lock particle");
                    particle.velocity *= *lambda;
                }
            }
            Thermostat::Custom {
                ..
            } => {
                todo!()
            }
        }
    }
}

