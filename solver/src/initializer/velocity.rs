use moldyn_core::{State, K_B, ParticleDatabase};
use rand_distr::{Distribution, Normal};

pub fn initialize_velocities_for_gas(state: &mut State, temperature: f64, particle_id: u16) {
    let mut rng = rand::thread_rng();
    let temperature = temperature * 0.01; // Scale temperature from Kelvin to program units
    let mass = ParticleDatabase::get_particle_mass(particle_id).expect("No particle in DB");
    let sigma = f64::sqrt(K_B * temperature / mass);
    let normal_distribution = Normal::new(0.0f64, sigma)
        .expect("Can't create normal distribution");
    let particles_count = state.particles[particle_id as usize].len();
    for i in 0..particles_count/2 {
        let x = normal_distribution.sample(&mut rng);
        let y = normal_distribution.sample(&mut rng);
        let z = normal_distribution.sample(&mut rng);
        {
            let particle = &mut state.particles[particle_id as usize][i];
            let mut particle = particle.write().expect("Can't lock particle");
            particle.velocity.x = x;
            particle.velocity.y = y;
            particle.velocity.z = z;
        }
        {
            let particle = &mut state.particles[particle_id as usize][i + particles_count / 2];
            let mut particle = particle.write().expect("Can't lock particle");
            particle.velocity.x = -x;
            particle.velocity.y = -y;
            particle.velocity.z = -z;
        }
    }
}
