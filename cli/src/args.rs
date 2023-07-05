use std::path::PathBuf;
use clap::{Parser, Subcommand};
use clap::ValueEnum;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// path to file with particles data
    #[arg(short = 'f', long)]
    pub file: PathBuf,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, ValueEnum)]
pub enum CrystalCellType {
    /// uniform grid like in gases
    Uniform,
}

#[derive(Clone, ValueEnum)]
pub enum IntegratorChoose {
    /// I think you know what it is
    VerletMethod,
    /// Custom method
    Custom,
}

#[derive(Clone)]
pub enum PotentialChoose {
    /// default 12-6 potential
    LennardJones,
    /// Custom potential
    Custom,
}



#[derive(Subcommand)]
pub enum Commands {
    /// initialize particles state
    Initialize {
        /// type of crystal cell
        #[arg(short = 't', long, value_enum)]
        crystal_cell_type: CrystalCellType,
        /// size of this cell
        #[arg(short = 's', long, num_args = 3, value_delimiter = ' ')]
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
        #[arg(short = 'l', long)]
        lattice_cell: f64,
        /// temperature in Kelvin
        #[arg(short = 'T', long)]
        temperature: f64,
    },
    /// run solver on particle state
    Solve {
        /// file for output
        #[arg(short = 'o', long)]
        out_file: PathBuf,
        /// method of integration
        #[arg(short = 'i', long)]
        integrate_method: IntegratorChoose,
        /// if integrate method is custom, this parameter must be set
        #[arg(long)]
        custom_method: Option<String>,
        /// file with potentials for any id pair
        #[arg(short = 'p', long)]
        potentials_file: Option<PathBuf>,
    },
}
