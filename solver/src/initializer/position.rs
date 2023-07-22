use moldyn_core::{Particle, ParticleDatabase, State};
use na::Vector3;
use std::sync::RwLock;

/// Particle creation errors
/// * particle ID didn't found
/// * too big
/// * out of boundary
#[derive(Eq, PartialEq, Debug)]
pub enum InitError {
    ParticleIdDidNotFound,
    TooBig,
    OutOfBoundary,
}

/// Creates start state with zero position and velocity.
///
/// # Arguments
///
/// * `number_particles` - slice with amounts of particles. `number_particles[i]` is amount of
/// particles with id=`i`
/// * `boundary` - boundary conditions vector
///
/// # Returns
/// State if there is no errors else returns [InitError]
pub fn initialize_particles(number_particles: &[usize], boundary: &Vector3<f64>) -> Result<State, InitError> {
    let mut particles: Vec<Vec<RwLock<Particle>>> = vec![];
    for i in 0..number_particles.len() {
        let mut particle_type = vec![];
        let particle_type_id = i as u16;
        if ParticleDatabase::get_particle_mass(particle_type_id).is_none() {
            return Err(InitError::ParticleIdDidNotFound);
        }
        for _ in 0..number_particles[i] {
            particle_type.push(RwLock::new(
                Particle::new(particle_type_id,
                              Vector3::new(0.0, 0.0, 0.0),
                              Vector3::new(0.0, 0.0, 0.0))
                    .unwrap()));
        }
        particles.push(particle_type);
    }
    Ok(State {
        particles,
        boundary_box: boundary.clone(),
    })
}

/// Initialize particles position on uniform grid of size `grid_size` in `start_position`
/// with cell size `unit_cell_size` for particle with id `particle_id` in `state`.
pub fn initialize_particles_position(
    state: &mut State,
    particle_id: u16,
    start_position: (f64, f64, f64),
    grid_size: (usize, usize, usize),
    unit_cell_size: f64,
) -> Result<(), InitError> {
    if ParticleDatabase::get_particle_mass(particle_id).is_none() {
        return Err(InitError::ParticleIdDidNotFound);
    }
    if grid_size.0 * grid_size.1 * grid_size.2 > state.particles[particle_id as usize].len() {
        return Err(InitError::TooBig);
    }
    if grid_size.0 as f64 * unit_cell_size > state.boundary_box.x
        || grid_size.1 as f64 * unit_cell_size > state.boundary_box.y
        || grid_size.2 as f64 * unit_cell_size > state.boundary_box.z
    {
        return Err(InitError::OutOfBoundary);
    }
    for x in 0..grid_size.0 {
        for y in 0..grid_size.1 {
            for z in 0..grid_size.2 {
                let particle = state.particles[particle_id as usize]
                    [x * grid_size.1 * grid_size.2 + y * grid_size.2 + z]
                    .get_mut()
                    .expect("Can't lock particle");
                particle.position = Vector3::new(
                    start_position.0 + x as f64 * unit_cell_size,
                    start_position.1 + y as f64 * unit_cell_size,
                    start_position.2 + z as f64 * unit_cell_size,
                );
            }
        }
    }
    Ok(())
}
