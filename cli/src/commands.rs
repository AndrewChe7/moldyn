use std::path::PathBuf;
use indicatif::{ProgressBar, ProgressStyle};
use nalgebra::Vector3;
use moldyn_core::{DataFile, MacroParameterType, ParticleDatabase};
use moldyn_solver::solver::{Integrator, update_force};
use crate::args::{CrystalCellType, IntegratorChoose};


const PROGRESS_BAR_SYMBOLS: &str = "█▉▊▋▌▍▎▏  ";
const PROGRESS_BAR_STYLE: &str = "{prefix:.bold}▕{wide_bar:.red}▏{pos:>7}/{len:7}";

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
    update_force(&mut state);
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
    pb.set_style(
        ProgressStyle::with_template(&PROGRESS_BAR_STYLE)
            .expect("Can't set style for progress bar")
            .progress_chars(PROGRESS_BAR_SYMBOLS)
    );
    pb.set_prefix("Solving steps: ");
    for _ in 0..*iteration_count {
        integrator.calculate(&mut state, *delta_time);
        data.add_state(&state);
        pb.inc(1);
    }
    pb.finish_with_message(format!("Calculated. States saved to {}", out_file.to_string_lossy()));
    data.save_to_file(out_file);
}

pub fn solve_macro(in_file: &PathBuf,
                   out_file: &PathBuf,
                   kinetic_energy: bool,
                   potential_energy: bool,
                   thermal_energy: bool,
                   temperature: bool,
                   pressure: bool,
                   custom: bool,
                   _custom_name: &Option<String>,
                   range: &Option<Vec<usize>>) {
    let mut data = DataFile::load_from_file(in_file);
    ParticleDatabase::load(&data.particles_database);
    let mut start = 0usize;
    let mut end = data.start_frame + data.frame_count;
    let mut step = 1usize;
    range.as_ref().and_then(|r| {
        if r.len() > 0 {
            start = r[0];
        }
        if r.len() > 1 {
            end = r[1];
        }
        if r.len() > 2 {
            step = r[2];
        }
        Some(0)
    });
    if start < data.start_frame {
        panic!("First frame is {}, but you've passed {}", data.start_frame, start);
    }
    if end > data.start_frame + data.frame_count {
        panic!("Last frame is {}, but you've passed {}", data.start_frame + data.frame_count, end);
    }
    let pb = ProgressBar::new((end - start) as u64);
    pb.set_style(
        ProgressStyle::with_template(&PROGRESS_BAR_STYLE)
            .expect("Can't set style for progress bar")
            .progress_chars(PROGRESS_BAR_SYMBOLS)
    );
    pb.set_prefix("Solving macro steps: ");
    for i in (start..end).step_by(step) {
        let state = data.frames.get(&i)
            .expect(format!("No frame with number {}, is it good??? Have you edited file?", i).as_str());
        let mut state: moldyn_core::State = state.into();
        let particle_count = state.particles.len();
        update_force(&mut state);
        let mut parameters = vec![];
        if kinetic_energy {
            let value = moldyn_solver::macro_parameters::get_kinetic_energy(&state, 0, particle_count);
            parameters.push(MacroParameterType::KineticEnergy(value));
        }
        if potential_energy {
            let value = moldyn_solver::macro_parameters::get_potential_energy(&state, 0, particle_count);
            parameters.push(MacroParameterType::PotentialEnergy(value));
        }
        let mass_velocity =
        if thermal_energy || temperature || pressure {
            moldyn_solver::macro_parameters::get_center_of_mass_velocity(&state, 0, particle_count)
        } else {
            Vector3::zeros()
        };
        let thermal_energy_value =
        if thermal_energy || temperature {
            let value = moldyn_solver::macro_parameters::get_thermal_energy(&state, 0, particle_count, &mass_velocity);
            parameters.push(MacroParameterType::ThermalEnergy(value));
            value
        } else {
            0.0
        };
        if temperature {
            let value = moldyn_solver::macro_parameters::get_temperature(thermal_energy_value, particle_count);
            parameters.push(MacroParameterType::Temperature(value));
        }
        if pressure {
            let value = moldyn_solver::macro_parameters::get_pressure(&state, 0, particle_count, &mass_velocity);
            parameters.push(MacroParameterType::Pressure(value));
        }
        if custom {
            todo!()
        }
        data.add_macro_params(i, &parameters);
        pb.inc(step as u64);
    }
    pb.finish_with_message(format!("Calculated. Macro Parameters saved to {}", out_file.to_string_lossy()));
    data.save_to_file(out_file);
}