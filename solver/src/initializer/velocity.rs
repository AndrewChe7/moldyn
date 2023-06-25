use moldyn_core::{State, K_B};
use rand_distr::{Distribution, Normal};


pub fn initialize_velocities_for_gas (state: &mut State, temperature: f64, mass: f64) {
    let mut rng = rand::thread_rng();
    let sigma = f64::sqrt(K_B * temperature / mass);
    let normal_distribution = Normal::new(0.0f64, sigma)
        .expect("Can't create normal distribution");
    for particle in &mut state.particles {
        let x = normal_distribution.sample(&mut rng);
        let y = normal_distribution.sample(&mut rng);
        let z = normal_distribution.sample(&mut rng);
        particle.velocity.x = x;
        particle.velocity.y = y;
        particle.velocity.z = z;
    }

}