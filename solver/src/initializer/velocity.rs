use rand::Rng;
use moldyn_core::{State, K_B, ParticleDatabase};
use rand_distr::StandardNormal;

/// Setup velocities according to Maxwell–Boltzmann distribution
pub fn initialize_velocities_maxwell_boltzmann(state: &mut State, temperature: f64, particle_id: u16) {
    let mut rng = rand::thread_rng();
    let temperature = temperature * 0.01; // Scale temperature from Kelvin to program units
    let mass = ParticleDatabase::get_particle_mass(particle_id).expect("No particle in DB");
    let sigma = f64::sqrt(K_B * temperature / mass);
    let particles_count = state.particles[particle_id as usize].len();
    for i in 0..particles_count/2 {
        let x = sigma * rng.sample::<f64, _>(StandardNormal);
        let y = sigma * rng.sample::<f64, _>(StandardNormal);
        let z = sigma * rng.sample::<f64, _>(StandardNormal);
        {
            let particle = &mut state.particles[particle_id as usize][i];
            particle.velocity.x = x;
            particle.velocity.y = y;
            particle.velocity.z = z;
        }
        {
            let particle = &mut state.particles[particle_id as usize][i + particles_count / 2];
            particle.velocity.x = -x;
            particle.velocity.y = -y;
            particle.velocity.z = -z;
        }
    }
}
