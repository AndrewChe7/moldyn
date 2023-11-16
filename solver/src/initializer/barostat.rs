use crate::macro_parameters::{get_center_of_mass_velocity, get_pressure};

/// Barostat enum object
pub enum Barostat {
    /// Paper: <https://pure.rug.nl/ws/files/64380902/1.448118.pdf>
    Berendsen {
        beta: f64,
        tau: f64,
        myu: f64,
    },
    /// Doesn't implemented yet
    Custom {
        name: String,
        custom_data: Vec<f64>,
    }
}


impl Barostat {
    /// Calculate resize coefficient
    pub fn calculate_myu (&mut self, state: &moldyn_core::State, delta_time: f64,
                          particle_type_id: u16, target_pressure: f64) {
        let mv = get_center_of_mass_velocity(&state, particle_type_id);
        let pressure = get_pressure(&state, particle_type_id, &mv);
        match self {
            Barostat::Berendsen {
                beta, tau, myu
            } => {
                let myu_cubed = 1.0 + delta_time * *beta / *tau * (pressure - target_pressure);
                *myu = myu_cubed.cbrt();
            }
            Barostat::Custom { .. } => {
                todo!()
            }
        }
    }

    /// Resize current state
    pub fn update(&self, state: &mut moldyn_core::State, _delta_time: f64,
                  particle_type_id: u16, _target_pressure: f64) {
        match self {
            Barostat::Berendsen {
                myu, ..
            } => {
                state.boundary_box *= *myu;
                for particle in state.particles[particle_type_id as usize].iter_mut() {
                    particle.position *= *myu;
                }
            }
            Barostat::Custom {
                ..
            } => {
                todo!()
            }
        }
    }
}