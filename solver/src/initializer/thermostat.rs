use crate::macro_parameters::{get_center_of_mass_velocity, get_temperature, get_thermal_energy};

/// Thermostat enum object
pub enum Thermostat {
    /// Paper: <https://pure.rug.nl/ws/files/64380902/1.448118.pdf>
    Berendsen {
        tau: f64,
        lambda: f64,
    },
    NoseHoover {
        tau: f64,
        psi: f64,
        lambda: f64,
    },
    /// Doesn't implemented yet
    Custom {
        name: String,
        custom_data: Vec<f64>,
    }
}

impl Thermostat {
    /// Calculate velocity scaling coefficient
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
            Thermostat::NoseHoover { tau,  psi, lambda} => {
                let psi_dot = - ((target_temperature / temperature) - 1.0) / *tau;
                *psi += psi_dot * (delta_time / 2.0);
                *lambda = f64::exp(-*psi * delta_time / 2.0);
            }
            Thermostat::Custom { .. } => {
                todo!()
            }
        }
    }

    /// Scale velocity
    pub fn update(&mut self, state: &mut moldyn_core::State, delta_time: f64,
                  particle_type_id: u16, target_temperature: f64) {
        let particles_count = state.particles[particle_type_id as usize].len();
        let mv = get_center_of_mass_velocity(&state, particle_type_id);
        let thermal_energy = get_thermal_energy(&state, particle_type_id, &mv);
        let temperature = get_temperature(thermal_energy, particles_count);
        match self {
            Thermostat::Berendsen {lambda, ..} => {
                state.particles[particle_type_id as usize].iter_mut().for_each(|particle| {
                    particle.velocity *= *lambda;
                });
            }
            Thermostat::NoseHoover { tau, lambda, psi } => {
                state.particles[particle_type_id as usize].iter_mut().for_each(|particle| {
                    particle.velocity *= *lambda;
                });
                let psi_dot = - ((target_temperature / temperature) - 1.0) / *tau;
                *psi += psi_dot * (delta_time / 2.0);
            }
            Thermostat::Custom {
                ..
            } => {
                todo!()
            }
        }
    }
}

