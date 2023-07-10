use moldyn_core::{State, K_B};
use rand_distr::{Distribution, Normal};

pub fn initialize_velocities_for_gas(state: &mut State, temperature: f64, mass: f64) {
    let mut rng = rand::thread_rng();
    let temperature = temperature * 0.01; // Scale temperature from Kelvin to program units
    let sigma = f64::sqrt(K_B * temperature / mass);
    let normal_distribution = Normal::new(0.0f64, sigma).expect("Can't create normal distribution");
    let particles_count = state.particles.len();
    for i in 0..particles_count/2 {
        let x = normal_distribution.sample(&mut rng);
        let y = normal_distribution.sample(&mut rng);
        let z = normal_distribution.sample(&mut rng);
        {
            let particle = state.particles.get_mut(i).expect("Can't get particle");
            let particle = particle.get_mut().expect("Can't lock particle");
            particle.velocity.x = x;
            particle.velocity.y = y;
            particle.velocity.z = z;
        }
        {
            let particle = state.particles.get_mut(i + particles_count/2).expect("Can't get particle");
            let particle = particle.get_mut().expect("Can't lock particle");
            particle.velocity.x = -x;
            particle.velocity.y = -y;
            particle.velocity.z = -z;
        }
    }
}
