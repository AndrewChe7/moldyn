use std::sync::Mutex;
use na::Vector3;
use moldyn_core::{State, Particle, ParticleDatabase};

#[derive(Eq, PartialEq, Debug)]
pub enum InitError {
    ParticleIdDidNotFound,
    TooBig,
    OutOfBoundary,
}

pub fn initialize_particles (number_particles: usize, boundary: &Vector3<f64>) -> State {
    let mut particles: Vec<Mutex<Particle>> = vec![];
    for _ in 0..number_particles {
        particles.push(Mutex::new(Particle::default()));
    }
    State {particles, boundary_box: boundary.clone()}
}

/// Initialize particles position on uniform grid of size [grid_size] in [start_position]
/// with cell size [unit_cell_size] starting from [first_particle] in [state].
pub fn initialize_particles_position(state: &mut State,
                                     first_particle: usize,
                                     particle_id: u16,
                                     start_position: (f64, f64, f64),
                                     grid_size: (usize, usize, usize),
                                     unit_cell_size: f64) -> Result<(), InitError> {
    if ParticleDatabase::get_particle_mass(particle_id).is_none() {
        return Err(InitError::ParticleIdDidNotFound);
    }
    if first_particle + grid_size.0 * grid_size.1 * grid_size.2 > state.particles.len() {
        return Err(InitError::TooBig);
    }
    if grid_size.0 as f64 * unit_cell_size > state.boundary_box.x ||
        grid_size.1 as f64 * unit_cell_size > state.boundary_box.y ||
        grid_size.2 as f64 * unit_cell_size > state.boundary_box.z {
        return Err(InitError::OutOfBoundary);
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
