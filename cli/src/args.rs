use std::path::PathBuf;
use clap::{Parser, Subcommand};
use clap::ValueEnum;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// path to file with particles data
    #[arg(short, long)]
    pub file: PathBuf,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, ValueEnum)]
pub enum CrystalCellType {
    /// uniform grid like in gases
    Uniform,
}

#[derive(Subcommand)]
pub enum Commands {
    /// initialize particles state
    Initialize {
        /// path to file
        #[arg(short, long)]
        crystal_cell_type: CrystalCellType,
        /// size of this cell
        #[arg(short, long, num_args = 3, value_delimiter = ' ')]
        size: Vec<u32>,
        /// name of particle to initialize
        #[arg(long)]
        particle_name: String,
        /// mass of particle to initialize
        #[arg(long)]
        particle_mass: f64,
        /// radius of particle to initialize
        #[arg(long)]
        particle_radius: f64,
        /// lattice cell
        #[arg(short, long)]
        lattice_cell: f64,
        /// temperature in Kelvin
        #[arg(short, long)]
        temperature: f64,
    },
}
