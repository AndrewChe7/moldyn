use std::path::PathBuf;
use clap::{Parser, Subcommand};
use clap::ValueEnum;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// path to file with particles data
    #[arg(short = 'f', long)]
    pub file: PathBuf,
    /// outputs in human-readable format
    #[arg(long, default_value_t=false)]
    pub pretty_print: bool,
    /// how often to make backup
    #[arg(short, long, default_value_t=2000)]
    pub backup: usize,
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
    /// https://doi.org/10.1103/PhysRev.159.98
    VerletMethod,
    /// Custom method
    Custom,
}

#[derive(Clone, ValueEnum)]
pub enum BarostatChoose {
    /// https://pure.rug.nl/ws/files/64380902/1.448118.pdf
    Berendsen,
    /// Custom method
    Custom,
}

#[derive(Clone, ValueEnum)]
pub enum ThermostatChoose {
    /// https://pure.rug.nl/ws/files/64380902/1.448118.pdf
    Berendsen,
    /// Custom method
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
        #[arg(short = 'n', long)]
        particle_name: String,
        /// mass of particle to initialize
        #[arg(short = 'm', long)]
        particle_mass: f64,
        /// radius of particle to initialize
        #[arg(short = 'r', long)]
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
        /// Barostat type
        #[arg(long)]
        barostat: Option<BarostatChoose>,
        /// Barostat parameters
        #[arg(long, num_args = 1..5, value_delimiter = ' ')]
        barostat_params: Option<Vec<f64>>,
        /// Barostat target pressure (Pa)
        #[arg(short = 'P', long)]
        pressure: Option<f64>,
        /// Thermostat type
        #[arg(long)]
        thermostat: Option<ThermostatChoose>,
        /// Thermostat parameters
        #[arg(long, num_args = 1..5, value_delimiter = ' ')]
        thermostat_params: Option<Vec<f64>>,
        /// Thermostat target temperature (in K)
        #[arg(short = 'T', long)]
        temperature: Option<f64>,
        /// file with potentials for any id pair
        #[arg(short = 'p', long)]
        potentials_file: Option<PathBuf>,
        /// how much iterations to count
        #[arg(short = 'c', long)]
        iteration_count: usize,
        /// how long each iteration should take
        #[arg(short = 't', long)]
        delta_time: f64,
    },
    /// calculate macro parameters for solved state
    SolveMacroParameters {
        /// file for output
        #[arg(short = 'o', long)]
        out_file: PathBuf,
        #[arg(short = 'k', long)]
        kinetic_energy: bool,
        #[arg(short = 'p', long)]
        potential_energy: bool,
        #[arg(short = 't', long)]
        thermal_energy: bool,
        #[arg(short = 'T', long)]
        temperature: bool,
        #[arg(short = 'P', long)]
        pressure: bool,
        /// if you use custom macro parameter, set it true
        #[arg(short = 'c', long)]
        custom: bool,
        /// if you set custom to true, set the name of it
        #[arg(short = 'C', long)]
        custom_name: Option<String>,
        /// Calculate all macro parameters
        #[arg(short = 'A', long)]
        all: bool,
        /// Which frames should be processed ([start]:[end]:[step])
        #[arg(short = 'r', long, num_args = 1..3, value_delimiter = ':')]
        range: Option<Vec<usize>>,
    },
    CheckImpulse
}
