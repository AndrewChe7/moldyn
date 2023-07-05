use std::path::PathBuf;
use nalgebra::Vector3;
use moldyn_core::{ParticleDatabase, save_data_to_file};
use crate::args::CrystalCellType;

pub fn initialize_uniform(file: &PathBuf,
                          size: &Vec<u32>,
                          particle_name: &String,
                          particle_mass: &f64,
                          particle_radius: &f64,
                          lattice_cell: &f64,
                          temperature: &f64) {
    ParticleDatabase::add(0, particle_name.as_str(), particle_mass.clone(), particle_radius.clone());
    let particles_count = (size[0] * size[1] * size[2]) as usize;
    let boundary_box = Vector3::new(
                                    lattice_cell * size[0] as f64,
                                    lattice_cell * size[1] as f64,
                                    lattice_cell * size[2] as f64);
    let mut state = moldyn_solver::initializer::initialize_particles(
        particles_count, &boundary_box);
    let res = moldyn_solver::initializer::initialize_particles_position(&mut state,
                                                              0, 0,
                                                              (0.0, 0.0, 0.0),
                                                              (size[0] as _, size[1] as _, size[2] as _),
                                                                        lattice_cell.clone());
    res.expect("Can't init positions");
    moldyn_solver::initializer::initialize_velocities_for_gas(&mut state,
                                                              temperature.clone(), particle_mass.clone());
    save_data_to_file(&state, file);
}

pub fn initialize(file: &PathBuf,
                  crystal_cell_type: &CrystalCellType,
                  size: &Vec<u32>,
                  particle_name: &String,
                  particle_mass: &f64,
                  particle_radius: &f64,
                  lattice_cell: &f64,
                  temperature: &f64) {
    match crystal_cell_type {
        CrystalCellType::Uniform => {
            initialize_uniform(file, size,
                               particle_name, particle_mass, particle_radius,
                               lattice_cell, temperature);
        }
    }
}

