use clap::Parser;
use crate::args::*;
use crate::commands::{check_impulse, initialize, solve, solve_macro};

mod args;
mod commands;

#[cfg(test)]
mod tests;

fn main() {
    env_logger::init();
    let args = Args::parse();
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
            out_file,
            integrate_method,
            custom_method,
            potentials_file,
            iteration_count,
            delta_time,
            thermostat,
            thermostat_params,
            temperature,
            barostat,
            barostat_params,
            pressure,
        } => {
            solve(&args.file, out_file, integrate_method,
                  custom_method, potentials_file, *iteration_count,
                  delta_time, args.backup,
                  thermostat, thermostat_params, temperature,
                  barostat, barostat_params, pressure);
        }
        Commands::SolveMacroParameters {
            out_file,
            kinetic_energy,
            potential_energy,
            thermal_energy,
            temperature,
            pressure,
            custom,
            custom_name,
            all,
            range
        } => {
            if *all {
                solve_macro(&args.file, out_file, true, true,
                            true, true, true,
                            *custom, custom_name, range);
            } else {
                solve_macro(&args.file, out_file, *kinetic_energy, *potential_energy,
                            *thermal_energy, *temperature, *pressure,
                            *custom, custom_name, range);
            }
        }
        Commands::CheckImpulse => {
            check_impulse(&args.file);
        }
    }
}
