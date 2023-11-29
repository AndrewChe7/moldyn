use std::time::Instant;
use clap::Parser;
use crate::args::*;
use crate::commands::{add_potential_to_file, check_impulse, generate_default_potentials, generate_histogram, initialize, particle_count, solve, solve_macro};

mod args;
mod commands;

#[cfg(test)]
mod tests;

fn main() {
    env_logger::init();
    let args = Args::parse();
    let start = Instant::now();
    match &args.command {
        Commands::Initialize {
            crystal_cell_type,
            size,
            particle_name,
            particle_mass,
            particle_radius,
            lattice_cell,
            temperature
        } => {
            initialize(&args.file, crystal_cell_type, size, particle_name,
                       particle_mass, particle_radius, lattice_cell, temperature);
        }
        Commands::Solve {
            state_number,
            integrate_method,
            threads_count,
            custom_method,
            use_potentials,
            iteration_count,
            delta_time,
            thermostat,
            thermostat_params,
            temperature,
            barostat,
            barostat_params,
            pressure,
        } => {
            if let Some(threads_count) = threads_count {
                rayon::ThreadPoolBuilder::new()
                    .num_threads(*threads_count)
                    .build_global().unwrap();
            }
            solve(&args.file, *state_number, integrate_method,
                  custom_method, use_potentials, *iteration_count,
                  delta_time,
                  thermostat, thermostat_params, temperature,
                  barostat, barostat_params, pressure);
        }
        Commands::SolveMacroParameters {
            kinetic_energy,
            potential_energy,
            thermal_energy,
            temperature,
            pressure,
            custom,
            custom_name,
            all,
            use_potentials,
        } => {
            if *all {
                solve_macro(&args.file,true, true,
                            true, true, true,
                            *custom, custom_name, use_potentials);
            } else {
                solve_macro(&args.file, *kinetic_energy, *potential_energy,
                            *thermal_energy, *temperature, *pressure,
                            *custom, custom_name, use_potentials);
            }
        }
        Commands::CheckImpulse => {
            check_impulse(&args.file);
        }
        Commands::ParticleCount => {
            particle_count(&args.file);
        }
        Commands::GenerateDefaultPotentials => {
            generate_default_potentials(&args.file);
        }
        Commands::SetPotential {
            particle_types,
            potential,
            params,
        } => {
            add_potential_to_file(&args.file, particle_types, potential, params);
        }
        Commands::GenerateVelocitiesHistogram {
            state_number,
            particle_types,
        } => {
            generate_histogram(&args.file, *state_number, &particle_types[..]);
        }
    }
    let duration = start.elapsed();
    if args.time {
        println!("Time elapsed: {}", duration.as_secs_f64());
    }
}
