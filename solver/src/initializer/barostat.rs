use crate::macro_parameters::{get_center_of_mass_velocity, get_pressure};

pub enum Barostat {
    Berendsen(f64),
    Custom {
        name: String,
        custom_data: Vec<f64>,
    }
}


impl Barostat {
    pub fn update(&self, state: &mut moldyn_core::State, delta_time: f64, target_pressure: f64) {
        let particles_count = state.particles.len();
        let mv = get_center_of_mass_velocity(&state, 0, particles_count);
        let pressure = get_pressure(&state, 0, particles_count, &mv);
        match self {
            Barostat::Berendsen(tau) => {
                let myu_cubed = 1.0 - delta_time / tau * (pressure - target_pressure);
                let myu = myu_cubed.cbrt();
                state.boundary_box *= myu;
                for particle in &mut state.particles {
                    let particle = particle.get_mut().expect("Can't lock particle");
                    particle.position *= myu;
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