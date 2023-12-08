use std::fs;
use std::fs::ReadDir;
use std::io::BufWriter;
use std::path::PathBuf;
use indicatif::{ProgressBar, ProgressStyle};
use nalgebra::Vector3;
use moldyn_core::{DataFileMacro, VectorData, MacroParameterType, ParticleDatabase, State, StateToSave, open_file_or_create};
use moldyn_solver::initializer::UnitCell;
use moldyn_solver::macro_parameters::get_momentum_of_system;
use moldyn_solver::solver::{Integrator, Potential, PotentialsDatabase, update_force};
use crate::args::{BarostatChoose, CrystalCellType, IntegratorChoose, PotentialChoose, ThermostatChoose};


const PROGRESS_BAR_SYMBOLS: &str = "█▉▊▋▌▍▎▏  ";
const PROGRESS_BAR_STYLE: &str = "{prefix:.bold}▕{wide_bar:.red}▏{pos:>7}/{len:7} {eta_precise:9} |";

pub fn backup_macro(data: &mut DataFileMacro, out_file: &PathBuf) {
    data.save_to_file(&out_file);
    data.reset_old();
}

pub fn generate_default_potentials(file: &PathBuf) {
    let mut potentials_database = PotentialsDatabase::new();
    potentials_database.set_potential(0, 0, Potential::new_lennard_jones(0.3418, 1.712));
    potentials_database.save_potentials_to_file(file);
}

pub fn add_potential_to_file(file: &PathBuf, particles: &Vec<u16>, potential: &PotentialChoose, params: &Vec<f64>) {
    let mut potentials_database = PotentialsDatabase::new();
    potentials_database.load_potentials_from_file(file);
    let potential = match potential {
        PotentialChoose::LennardJones => {
            Potential::new_lennard_jones(params[0], params[1])
        }
        PotentialChoose::Custom => {
            todo!()
        }
    };
    potentials_database.set_potential(particles[0], particles[1], potential);
    potentials_database.save_potentials_to_file(file);
}

pub fn initialize(file: &PathBuf,
                  crystal_cell_type: &CrystalCellType,
                  size: &Vec<u32>,
                  particle_name: &String,
                  particle_mass: &f64,
                  particle_radius: &f64,
                  lattice_cell: &f64,
                  temperature: &f64) {
    let unit_cell_type = match crystal_cell_type {
        CrystalCellType::U => UnitCell::U,
        CrystalCellType::FCC => UnitCell::FCC,
    };
    ParticleDatabase::add(0, particle_name.as_str(), particle_mass.clone(), particle_radius.clone());
    let particles_count = match unit_cell_type {
        UnitCell::U => {
            (size[0] * size[1] * size[2]) as usize
        }
        UnitCell::FCC => {
            (size[0] * size[1] * size[2] * 4) as usize
        }
    };
    let boundary_box = Vector3::new(
        lattice_cell * size[0] as f64,
        lattice_cell * size[1] as f64,
        lattice_cell * size[2] as f64);
    let mut state = moldyn_solver::initializer::initialize_particles(
        &[particles_count], &boundary_box).unwrap();
    let res = moldyn_solver::initializer::initialize_particles_position(
        unit_cell_type, &mut state,
        0, (0.0, 0.0, 0.0),
        (size[0] as _, size[1] as _, size[2] as _), lattice_cell.clone());
    res.expect("Can't init positions");
    moldyn_solver::initializer::initialize_velocities_maxwell_boltzmann(&mut state,
                                                              temperature.clone(), 0);
    let data = StateToSave::from(&state);
    data.save_to_file(file, 0);
    ParticleDatabase::save_particles_data(file).expect("Can't save particles database");
}

pub fn solve(file: &PathBuf,
             state_number: usize,
             integrator: &IntegratorChoose,
             _custom_method: &Option<String>,
             use_potentials: &bool,
             iteration_count: usize,
             delta_time: &f64,
             thermostat_choose: &Option<ThermostatChoose>,
             thermostat_params: &Option<Vec<f64>>,
             temperature: &Option<f64>,
             barostat_choose: &Option<BarostatChoose>,
             barostat_params: &Option<Vec<f64>>,
             pressure: &Option<f64>) {
    let data = StateToSave::load_from_file(file, state_number);
    ParticleDatabase::load_particles_data(file).expect("Can't load particle database");
    let mut potentials_database = PotentialsDatabase::new();
    let mut state = data.into();
    if *use_potentials {
        potentials_database.load_potentials_from_file(file);
    }
    update_force(&potentials_database, &mut state);
    let integrator = match integrator {
        IntegratorChoose::VerletMethod => {
            Integrator::VerletMethod
        }
        _ => {
            todo!()
        }
    };
    let mut thermostat = if let Some(thermostat_choose) = thermostat_choose {
        Some(match thermostat_choose {
                ThermostatChoose::Berendsen => {
                    moldyn_solver::initializer::Thermostat::Berendsen {
                        tau: thermostat_params.clone()
                            .expect("No thermostat parameters. Need tau for Berendsen")
                            [0],
                        lambda: 0.0,
                    }
                }
                ThermostatChoose::NoseHoover => {
                    moldyn_solver::initializer::Thermostat::NoseHoover {
                        tau: thermostat_params.clone()
                            .expect("No thermostat parameters. Need tau for Nose-Hoover")
                            [0],
                        psi: 0.0,
                        lambda: 0.0,
                    }
                }
                ThermostatChoose::Custom => {
                    todo!()
                }
        })
    } else {
        None
    };
    let mut barostat = if let Some(barostat_choose) = barostat_choose {
        Some(match barostat_choose {
                BarostatChoose::Berendsen => {
                    let params = barostat_params.clone()
                        .expect("No barostat parameters. Need beta and tau for Berendsen");
                    moldyn_solver::initializer::Barostat::Berendsen {
                        beta: params[0],
                        tau: params[1],
                        myu: 0.0,
                    }
                }
                BarostatChoose::Custom => {
                    todo!()
                }
            })
    } else {
        None
    };
    let pb = ProgressBar::new(iteration_count as u64);
    pb.set_style(
        ProgressStyle::with_template(&PROGRESS_BAR_STYLE)
            .expect("Can't set style for progress bar")
            .progress_chars(PROGRESS_BAR_SYMBOLS)
    );
    pb.set_prefix("Solving steps: ");
    let pressure = if barostat.is_some() {
        pressure.expect("No pressure was passed")
    } else {
        0.0
    };
    let temperature = if thermostat.is_some() {
        temperature.expect("No temperature was passed")
    } else {
        0.0
    };
    let mut barostat = if let Some(barostat) = &mut barostat {
        Some((barostat, pressure))
    } else {
        None
    };
    let mut thermostat = if let Some(thermostat) = &mut thermostat {
        Some((thermostat, temperature))
    } else {
        None
    };
    for i in 0..iteration_count {
        let data = StateToSave::from(&state);
        data.save_to_file(file, state_number + i);
        integrator.calculate(&potentials_database, &mut state, *delta_time, &mut barostat, &mut thermostat);
        pb.inc(1);
    }
    pb.finish_with_message("Calculated.");
    let data = StateToSave::from(&state);
    data.save_to_file(file, state_number + iteration_count);
}

fn get_last_path (paths: ReadDir) -> usize {
    paths.map(|path| {
        let path = path.unwrap();
        let i: usize = path.path().with_extension("")
            .file_name().unwrap()
            .to_str().unwrap()
            .parse().unwrap();
        i
    }).max().unwrap()
}

pub fn solve_macro(file: &PathBuf,
                   kinetic_energy: bool,
                   potential_energy: bool,
                   thermal_energy: bool,
                   temperature: bool,
                   pressure: bool,
                   custom: bool,
                   _custom_name: &Option<String>,
                   use_potentials: &bool) {
    let paths = fs::read_dir(file.join("data"))
        .expect("Can't read directory");
    let mut potentials_database = PotentialsDatabase::new();
    if *use_potentials {
        potentials_database.load_potentials_from_file(file);
    }
    let start = 0;
    let end = get_last_path(paths);
    let pb = ProgressBar::new((end - start) as u64);
    pb.set_style(
        ProgressStyle::with_template(&PROGRESS_BAR_STYLE)
            .expect("Can't set style for progress bar")
            .progress_chars(PROGRESS_BAR_SYMBOLS)
    );
    pb.set_prefix("Solving macro steps: ");
    let mut macro_data = DataFileMacro::new();
    ParticleDatabase::load_particles_data(file).expect("Can't load particle database");
    for i in start..=end {
        let state_data = StateToSave::load_from_file(file, i);
        let mut state: moldyn_core::State = state_data.into();
        let particle_count = state.particles.iter().map( |t| t.len() ).sum();
        update_force(&potentials_database, &mut state);
        let mut parameters = vec![];
        if kinetic_energy {
            let value = moldyn_solver::macro_parameters::get_kinetic_energy(&state, 0);
            parameters.push(MacroParameterType::KineticEnergy(value));
        }
        if potential_energy {
            let value = moldyn_solver::macro_parameters::get_potential_energy(&state, 0);
            parameters.push(MacroParameterType::PotentialEnergy(value));
        }
        let mass_velocity =
            if thermal_energy || temperature || pressure {
                moldyn_solver::macro_parameters::get_center_of_mass_velocity(&state, 0)
            } else {
                Vector3::zeros()
            };
        let thermal_energy_value =
            if thermal_energy || temperature {
                let value = moldyn_solver::macro_parameters::get_thermal_energy(&state, 0, &mass_velocity);
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
            let value = moldyn_solver::macro_parameters::get_pressure(&state, 0, &mass_velocity);
            parameters.push(MacroParameterType::Pressure(value));
        }
        if custom {
            todo!()
        }
        macro_data.add_macro_params(i, &parameters, particle_count);
        pb.inc(1);
    }
    backup_macro(&mut macro_data, &file.join("macro.csv"));
    pb.finish();
}

pub fn check_impulse (file: &PathBuf) {
    let paths = fs::read_dir(file.join("data"))
        .expect("Can't read directory");
    ParticleDatabase::load_particles_data(file).expect("Can't load particle database");
    let start = 0;
    let end = get_last_path(paths);
    let data = StateToSave::load_from_file(file, start);
    {
        println!("First frame");
        let state: State = data.into();
        for (particle_type, _) in state.particles.iter().enumerate() {
            let p = get_momentum_of_system(&state, particle_type as u16);
            let p_abs = p.magnitude();
            println!("type = {particle_type};|p| = {p_abs:.15};p = {p:.15}");
        }
    }
    let data = StateToSave::load_from_file(file, end);
    {
        println!("Last frame");
        let state: State = data.into();
        for (particle_type, _) in state.particles.iter().enumerate() {
            let p = get_momentum_of_system(&state, particle_type as u16);
            let p_abs = p.magnitude();
            println!("type = {particle_type};|p| = {p_abs:.15};p = {p:.15}");
        }
    }
}

pub fn particle_count(file: &PathBuf) {
    let data = StateToSave::load_from_file(file, 0);
    ParticleDatabase::load_particles_data(file).expect("Can't load particle database");
    let state: State = data.into();
    let count: usize = state.particles.iter().map(| type_data | {
        type_data.len()
    }).sum();
    println!("Particle count: {count}");
}

pub fn generate_histogram(in_file: &PathBuf,
                          state_number: usize,
                          particle_types: &[u16]) {
    let data = StateToSave::load_from_file(in_file, state_number);
    ParticleDatabase::load_particles_data(in_file).expect("Can't load particle database");
    let state: State = data.into();
    let mut hist_data = vec![];
    for particle_type in particle_types {
        for particle in &state.particles[*particle_type as usize] {
            let hist = VectorData {
                x: particle.velocity.x,
                y: particle.velocity.y,
                z: particle.velocity.z,
            };
            hist_data.push(hist);
        }
    }
    let out_file = in_file.join("hist.csv");
    let file = open_file_or_create(&out_file);
    let buf_writer = BufWriter::with_capacity(1073741824, file);
    let mut wtr = csv::Writer::from_writer(buf_writer);
    for value in hist_data.iter() {
        wtr.serialize(value.clone()).expect("Can't serialize data");
    }
    wtr.flush().expect("Can't write");
}