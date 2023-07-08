use clap::Parser;
use crate::args::*;
use crate::commands::{initialize, solve};

#[macro_use]
extern crate log;
mod args;
mod commands;

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
        } => {
            solve(&args.file, out_file, integrate_method,
                  custom_method, potentials_file, iteration_count, delta_time);
        }
    }
}
