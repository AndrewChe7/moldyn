use std::sync::Mutex;
use na::Vector3;
use moldyn_core::{State, Particle, ParticleDatabase};

#[derive(Eq, PartialEq, Debug)]
pub enum InitError {
    ParticleIdDidNotFound,
    TooBig
}

pub fn initialize_particles (number_particles: usize) -> State {
    let mut particles: Vec<Mutex<Particle>> = vec![];
    for _ in 0..number_particles {
        particles.push(Mutex::new(Particle::new()));
    }
    State {particles}
}

/// Initialize particles position on uniform grid of size [grid_size] in [start_position]
/// with cell size [unit_cell_size] starting from [first_particle] in [state].
pub fn initialize_particles_position(state: &mut State,
                                     first_particle: usize,
                                     particle_id: u16,
                                     start_position: (f64, f64, f64),
                                     grid_size: (usize, usize, usize),
                                     unit_cell_size: f64) -> Result<(), InitError> {
    if ParticleDatabase::get_particle_mass(particle_id) == None {
        return Err(InitError::ParticleIdDidNotFound);
    }
    if first_particle + grid_size.0 * grid_size.1 * grid_size.2 > state.particles.len() {
        return Err(InitError::TooBig);
    }
    for x in 0..grid_size.0 {
        for y in 0..grid_size.1 {
            for z in 0..grid_size.2 {
                let particle = state.particles[first_particle + x*grid_size.1*grid_size.2 + y*grid_size.2 + z].get_mut().expect("Can't lock particle");
                particle.position = Vector3::new(start_position.0 + x as f64 * unit_cell_size,
                                                 start_position.1 + y as f64 * unit_cell_size,
                                                 start_position.2 + z as f64 * unit_cell_size);
                particle.id = particle_id;
                particle.mass = ParticleDatabase::get_particle_mass(particle_id).unwrap();
            }
        }
    }
    Ok(())
}
