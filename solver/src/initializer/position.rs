use moldyn_core::{Particle, ParticleDatabase, State};
use na::Vector3;
use rand::prelude::*;

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

/// Unit cell types
pub enum UnitCell {
    /// Uniform
    U,
    /// Face-Centered Cubic
    FCC,
}

impl UnitCell {
    pub fn initialize_particles_position(
        self,
        state: &mut State,
        particle_id: u16,
        start_position: (f64, f64, f64),
        grid_size: (usize, usize, usize),
        unit_cell_size: f64,
    ) -> Result<(), InitError> {
        match self {
            UnitCell::U => {
                if grid_size.0 * grid_size.1 * grid_size.2 > state.particles[particle_id as usize].len() {
                    return Err(InitError::TooBig);
                }
                for x in 0..grid_size.0 {
                    for y in 0..grid_size.1 {
                        for z in 0..grid_size.2 {
                            let particle = &mut state.particles[particle_id as usize]
                                [x * grid_size.1 * grid_size.2 + y * grid_size.2 + z];
                            particle.position = Vector3::new(
                                start_position.0 + x as f64 * unit_cell_size,
                                start_position.1 + y as f64 * unit_cell_size,
                                start_position.2 + z as f64 * unit_cell_size,
                            );
                        }
                    }
                }
            }
            UnitCell::FCC => {
                if grid_size.0 * grid_size.1 * grid_size.2 * 4 > state.particles[particle_id as usize].len() {
                    return Err(InitError::TooBig);
                }
                for x in 0..grid_size.0 {
                    for y in 0..grid_size.1 {
                        for z in 0..grid_size.2 {
                            let cell_id = x * grid_size.1 * grid_size.2 + y * grid_size.2 + z;
                            {
                                let particle = &mut state.particles[particle_id as usize]
                                    [cell_id * 4];
                                particle.position = Vector3::new(
                                    start_position.0 + x as f64 * unit_cell_size,
                                    start_position.1 + y as f64 * unit_cell_size,
                                    start_position.2 + z as f64 * unit_cell_size,
                                );
                            }
                            {
                                let particle = &mut state.particles[particle_id as usize]
                                    [cell_id * 4 + 1];
                                particle.position = Vector3::new(
                                    start_position.0 + x as f64 * unit_cell_size,
                                    start_position.1 + (y as f64 + 0.5) * unit_cell_size,
                                    start_position.2 + (z as f64 + 0.5) * unit_cell_size,
                                );
                            }
                            {
                                let particle = &mut state.particles[particle_id as usize]
                                    [cell_id * 4 + 2];
                                particle.position = Vector3::new(
                                    start_position.0 + (x as f64 + 0.5) * unit_cell_size,
                                    start_position.1 + y as f64 * unit_cell_size,
                                    start_position.2 + (z as f64 + 0.5) * unit_cell_size,
                                );
                            }
                            {
                                let particle = &mut state.particles[particle_id as usize]
                                    [cell_id * 4 + 3];
                                particle.position = Vector3::new(
                                    start_position.0 + (x as f64 + 0.5) * unit_cell_size,
                                    start_position.1 + (y as f64 + 0.5) * unit_cell_size,
                                    start_position.2 + z as f64 * unit_cell_size,
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
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
    let mut particles: Vec<Vec<Particle>> = vec![];
    for i in 0..number_particles.len() {
        let mut particle_type = vec![];
        let particle_type_id = i as u16;
        if ParticleDatabase::get_particle_mass(particle_type_id).is_none() {
            return Err(InitError::ParticleIdDidNotFound);
        }
        for _ in 0..number_particles[i] {
            particle_type.push(
                Particle::new(particle_type_id,
                              Vector3::new(0.0, 0.0, 0.0),
                              Vector3::new(0.0, 0.0, 0.0))
                    .unwrap());
        }
        particles.push(particle_type);
    }
    Ok(State {
        particles,
        boundary_box: boundary.clone(),
    })
}

/// Method just for testing. Initializes random positions.
pub fn randomize_positions(state: &mut State,
                           particle_id: u16,
                           grid_size: (usize, usize, usize),
                           unit_cell_size: f64,) {
    let mut rng = StdRng::seed_from_u64(42);
    for particle in state.particles[particle_id as usize].iter_mut() {
        let x = rng.gen::<f64>() * grid_size.0 as f64 * unit_cell_size;
        let y = rng.gen::<f64>() * grid_size.1 as f64 * unit_cell_size;
        let z = rng.gen::<f64>() * grid_size.2 as f64 * unit_cell_size;
        particle.position = [x, y, z].into();
    }
}

/// Initialize particles position on  `unit_cell_type`-grid of size `grid_size` in `start_position`
/// with cell size `unit_cell_size` for particle with id `particle_id` in `state`.
pub fn initialize_particles_position(
    unit_cell_type: UnitCell,
    state: &mut State,
    particle_id: u16,
    start_position: (f64, f64, f64),
    grid_size: (usize, usize, usize),
    unit_cell_size: f64,
) -> Result<(), InitError> {
    if ParticleDatabase::get_particle_mass(particle_id).is_none() {
        return Err(InitError::ParticleIdDidNotFound);
    }
    if grid_size.0 as f64 * unit_cell_size > state.boundary_box.x
        || grid_size.1 as f64 * unit_cell_size > state.boundary_box.y
        || grid_size.2 as f64 * unit_cell_size > state.boundary_box.z
    {
        return Err(InitError::OutOfBoundary);
    }
    unit_cell_type.initialize_particles_position(state, particle_id, start_position,
                                                 grid_size, unit_cell_size)
}
