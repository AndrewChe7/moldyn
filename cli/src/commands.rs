use std::path::PathBuf;
use indicatif::ProgressBar;
use nalgebra::Vector3;
use moldyn_core::{DataFile, ParticleDatabase};
use moldyn_solver::solver::Integrator;
use crate::args::{CrystalCellType, IntegratorChoose};

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
    let data = DataFile::init_from_state(&state);
    data.save_to_file(file);
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

pub fn solve(in_file: &PathBuf,
             out_file: &PathBuf,
             integrator: &IntegratorChoose,
             _custom_method: &Option<String>,
             potentials_file: &Option<PathBuf>,
             iteration_count: &usize,
             delta_time: &f64) {
    let mut data = DataFile::load_from_file(in_file);
    ParticleDatabase::load(&data.particles_database);
    let mut state = data.get_last_frame();
    let integrator = match integrator {
        IntegratorChoose::VerletMethod => {
            Integrator::VerletMethod
        }
        _ => {
            todo!()
        }
    };
    if let Some(potentials_file) = potentials_file {
        moldyn_solver::solver::load_potentials_from_file(potentials_file);
    }
    let pb = ProgressBar::new(*iteration_count as u64);
    for _ in 0..*iteration_count {
        integrator.calculate(&mut state, *delta_time);
        data.add_state(&state);
        pb.inc(1);
    }
    pb.finish_with_message(format!("Calculated. States saved to {}", out_file.to_string_lossy()));
    data.save_to_file(out_file);
}
