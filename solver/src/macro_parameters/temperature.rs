use moldyn_core::K_B;

pub fn get_temperature(thermal_energy: f64, number_particles: usize) -> f64 {
    let t = (2.0 * thermal_energy) / (3.0 * number_particles as f64 * K_B);
    t * 100.0 // Convert from program units to Kelvin
}
