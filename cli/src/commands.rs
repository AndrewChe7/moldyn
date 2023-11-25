use std::fs;
use std::fs::{File, OpenOptions};
use std::io::BufWriter;
use std::path::PathBuf;
use indicatif::{ProgressBar, ProgressStyle};
use nalgebra::Vector3;
use moldyn_core::{DataFile, DataFileMacro, HistogramData, MacroParameterType, ParticleDatabase};
use moldyn_solver::initializer::UnitCell;
use moldyn_solver::macro_parameters::get_momentum_of_system;
use moldyn_solver::solver::{Integrator, load_potentials_from_file, Potential, save_potentials_to_file, set_potential, update_force};
use crate::args::{BarostatChoose, CrystalCellType, IntegratorChoose, PotentialChoose, ThermostatChoose};


const PROGRESS_BAR_SYMBOLS: &str = "█▉▊▋▌▍▎▏  ";
const PROGRESS_BAR_STYLE: &str = "{prefix:.bold}▕{wide_bar:.red}▏{pos:>7}/{len:7} {eta_precise:9} |";

pub fn backup(data: &mut DataFile, out_file: &PathBuf, iteration: usize, pretty_print: bool) {
    let mut backup_file = out_file.clone();
    backup_file.set_extension(format!("{}.json", iteration));
    data.save_to_file(&backup_file, pretty_print);
    data.reset_old();
}

pub fn backup_macro(data: &mut DataFileMacro, out_file: &PathBuf) {
    let mut backup_file = out_file.clone();
    backup_file.set_extension("macro.csv");
    data.save_to_file(&backup_file);
    data.reset_old();
}

pub fn generate_default_potentials(file: &PathBuf) {
    set_potential(0, 0, Potential::new_lennard_jones(0.3418, 1.712));
    save_potentials_to_file(file);
}

pub fn add_potential_to_file(file: &PathBuf, particles: &Vec<u16>, potential: &PotentialChoose, params: &Vec<f64>) {
    load_potentials_from_file(file);
    let potential = match potential {
        PotentialChoose::LennardJones => {
            Potential::new_lennard_jones(params[0], params[1])
        }
        PotentialChoose::Custom => {
            todo!()
        }
    };
    set_potential(particles[0], particles[1], potential);
    save_potentials_to_file(file);
}

pub fn initialize(file: &PathBuf,
                  crystal_cell_type: &CrystalCellType,
                  size: &Vec<u32>,
                  particle_name: &String,
                  particle_mass: &f64,
                  particle_radius: &f64,
                  lattice_cell: &f64,
                  temperature: &f64,
                  pretty_print: bool) {
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
    let data = DataFile::init_from_state(&state);
    data.save_to_file(file, pretty_print);
}

pub fn solve(in_file: &PathBuf,
             out_file: &PathBuf,
             integrator: &IntegratorChoose,
             _custom_method: &Option<String>,
             potentials_file: &Option<PathBuf>,
             iteration_count: usize,
             delta_time: &f64,
             backup_frequency: usize,
             thermostat_choose: &Option<ThermostatChoose>,
             thermostat_params: &Option<Vec<f64>>,
             temperature: &Option<f64>,
             barostat_choose: &Option<BarostatChoose>,
             barostat_params: &Option<Vec<f64>>,
             pressure: &Option<f64>,
             pretty_print: bool) {
    let mut data = DataFile::load_from_file(in_file);
    ParticleDatabase::load(&data.particles_database);
    let (last_frame, mut state) = data.get_last_frame();
    if let Some(potentials_file) = potentials_file {
        load_potentials_from_file(potentials_file);
    }
    update_force(&mut state);
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
    for i in last_frame..last_frame + iteration_count {
        if i > 0 && (i - last_frame) % backup_frequency == 0 {
            backup(&mut data, out_file, i, pretty_print);
        }
        let barostat = if let Some(barostat) = &mut barostat {
            Some((barostat, pressure))
        } else {
            None
        };
        let thermostat = if let Some(thermostat) = &mut thermostat {
            Some((thermostat, temperature))
        } else {
            None
        };
        integrator.calculate(&mut state, *delta_time, barostat, thermostat);
        data.add_state(&state);
        pb.inc(1);
    }
    pb.finish_with_message(format!("Calculated. States saved to {}", out_file.to_string_lossy()));
    backup(&mut data, out_file, last_frame + iteration_count, pretty_print);
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
    let file_dir = in_file.parent().expect("Can't get project folder");
    let paths = fs::read_dir(file_dir).expect("Can't read directory");
    let file_path_without_ext = in_file
        .with_extension("")
        .with_extension("");
    let mut sizes: Vec<usize> = vec![];
    for path in paths {
        let path = path.expect("Can't get file");
        let path = path.path();
        if path.with_extension("").with_extension("") == file_path_without_ext {
            if let Some(extension) = path.with_extension("").extension() {
                let extension_string = extension.to_str()
                    .expect(format!("Can't convert to str {:?}", extension).as_str());
                if let Ok(last) = extension_string.parse() {
                    sizes.push(last);
                }
            }
        }
    }
    let start = 0;
    let end = sizes.iter().max().unwrap();
    let pb = ProgressBar::new((end - start) as u64);
    pb.set_style(
        ProgressStyle::with_template(&PROGRESS_BAR_STYLE)
            .expect("Can't set style for progress bar")
            .progress_chars(PROGRESS_BAR_SYMBOLS)
    );
    pb.set_prefix("Solving macro steps: ");
    let mut macro_data = DataFileMacro::new();
    for size in sizes.iter() {
        let file = in_file.with_extension("").with_extension(format!("{}.json", size));
        let data = DataFile::load_from_file(&file);
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
        start = start.max(data.start_frame);
        end = end.min(data.start_frame + data.frame_count - 1);
        for i in (start..=end).step_by(step) {
            let state = data.frames.get(&i)
                .expect(format!("No frame with number {}, is it good??? Have you edited file?", i).as_str());
            let mut state: moldyn_core::State = state.into();
            let particle_count = state.particles.iter().map( |t| t.len() ).sum();
            update_force(&mut state);
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
            pb.inc(step as u64);
        }
    }
    backup_macro(&mut macro_data, out_file);
    pb.finish();
}

pub fn check_impulse (in_file: &PathBuf) {
    let file_dir = in_file.parent().expect("Can't get project folder");
    let paths = fs::read_dir(file_dir).expect("Can't read directory");
    let file_path_without_ext = in_file
        .with_extension("")
        .with_extension("");
    let mut sizes: Vec<usize> = vec![];
    for path in paths {
        let path = path.expect("Can't get file");
        let path = path.path();
        if path.with_extension("").with_extension("") == file_path_without_ext {
            if let Some(extension) = path.with_extension("").extension() {
                let extension_string = extension.to_str()
                    .expect(format!("Can't convert to str {:?}", extension).as_str());
                if let Ok(last) = extension_string.parse() {
                    sizes.push(last);
                }
            }
        }
    }
    let first = sizes.iter().min().unwrap().clone();
    let file = in_file.with_extension("").with_extension(format!("{}.json", first));
    let data = DataFile::load_from_file(&file);
    ParticleDatabase::load(&data.particles_database);
    {
        println!("First frame");
        let state = data.frames.get(&0).unwrap().into();
        for (particle_type, _) in state.particles.iter().enumerate() {
            let p = get_momentum_of_system(&state, particle_type as u16);
            let p_abs = p.magnitude();
            println!("type = {particle_type};|p| = {p_abs:.15};p = {p:.15}");
        }
    }
    let last = sizes.iter().max().unwrap().clone();
    let file = in_file.with_extension("").with_extension(format!("{}.json", last));
    let data = DataFile::load_from_file(&file);
    ParticleDatabase::load(&data.particles_database);
    {
        println!("Last frame");
        let (_, state) = data.get_last_frame();
        for (particle_type, _) in state.particles.iter().enumerate() {
            let p = get_momentum_of_system(&state, particle_type as u16);
            let p_abs = p.magnitude();
            println!("type = {particle_type};|p| = {p_abs:.15};p = {p:.15}");
        }
    }
}

pub fn particle_count(in_file: &PathBuf) {
    let data = DataFile::load_from_file(&in_file);
    ParticleDatabase::load(&data.particles_database);
    let state = data.frames.get(&0).unwrap();
    let count: usize = state.particles.iter().map(|(_, type_data)| {
        type_data.len()
    }).sum();
    println!("Particle count: {count}");
}

pub fn generate_histogram(in_file: &PathBuf, out_file: &PathBuf, step: usize, particle_types: &[u16]) {
    let file_dir = in_file.parent().expect("Can't get project folder");
    let paths = fs::read_dir(file_dir).expect("Can't read directory");
    let file_path_without_ext = in_file
        .with_extension("")
        .with_extension("");
    let mut sizes: Vec<usize> = vec![];
    for path in paths {
        let path = path.expect("Can't get file");
        let path = path.path();
        if path.with_extension("").with_extension("") == file_path_without_ext {
            if let Some(extension) = path.with_extension("").extension() {
                let extension_string = extension.to_str()
                    .expect(format!("Can't convert to str {:?}", extension).as_str());
                if let Ok(last) = extension_string.parse() {
                    sizes.push(last);
                }
            }
        }
    }
    sizes.sort();
    let res = sizes.binary_search(&step);
    let file_number = match res {
        Ok(x) => {
            if x > sizes.len() {
                panic!("No such step!")
            }
            sizes[x]
        }
        Err(x) => {
            sizes[x]
        }
    };
    let file = in_file.with_extension("").with_extension(format!("{}.json", file_number));
    let data = DataFile::load_from_file(&file);
    let state = data.frames.get(&step).unwrap();
    let mut hist_data = vec![];
    for particle_type in particle_types {
        for (_, particle) in &state.particles[particle_type] {
            let hist = HistogramData {
                x: particle.velocity.x,
                y: particle.velocity.y,
                z: particle.velocity.z,
                abs: particle.velocity.magnitude(),
            };
            hist_data.push(hist);
        }
    }
    let file = if !out_file.exists() {
        File::create(out_file)
    } else {
        OpenOptions::new().truncate(true).write(true).open(out_file)
    }.expect("Can't write to file");
    let buf_writer = BufWriter::with_capacity(1073741824, file);
    let mut wtr = csv::Writer::from_writer(buf_writer);
    for value in hist_data.iter() {
        wtr.serialize(value.clone()).expect("Can't serialize data");
    }
    wtr.flush().expect("Can't write");
}