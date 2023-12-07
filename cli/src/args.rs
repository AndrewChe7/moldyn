use std::path::PathBuf;
use clap::{Parser, Subcommand};
use clap::ValueEnum;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// path to file with particles data
    #[arg(short = 'f', long)]
    pub file: PathBuf,
    /// Measure time of work
    #[arg(long, default_value_t=false)]
    pub time: bool,
    /// how often to save state
    #[arg(long, default_value_t=1)]
    pub frames_per_save: usize,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, ValueEnum)]
pub enum CrystalCellType {
    /// uniform grid like in gases
    U,
    /// Face-Centered Cubic grid like in metals.
    FCC,
}

#[derive(Clone, ValueEnum)]
pub enum IntegratorChoose {
    /// <https://doi.org/10.1103/PhysRev.159.98>
    VerletMethod,
    /// Custom method (doesn't implemented yet)
    Custom,
}

#[derive(Clone, ValueEnum)]
pub enum BarostatChoose {
    /// <https://pure.rug.nl/ws/files/64380902/1.448118.pdf>
    Berendsen,
    /// Custom method (doesn't implemented yet)
    Custom,
}

#[derive(Clone, ValueEnum)]
pub enum ThermostatChoose {
    /// <https://pure.rug.nl/ws/files/64380902/1.448118.pdf>
    Berendsen,
    NoseHoover,
    /// Custom method (doesn't implemented yet)
    Custom,
}

#[derive(Clone, ValueEnum)]
pub enum PotentialChoose {
    LennardJones,
    Custom,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate file with default potentials
    GenerateDefaultPotentials,
    /// Sets potential between different particle types
    SetPotential {
        #[arg(short = 'i', long, num_args = 2, value_delimiter = ' ')]
        particle_types: Vec<u16>,
        #[arg(short = 'p', long, value_enum)]
        potential: PotentialChoose,
        #[arg(long, num_args = 1.., value_delimiter = ' ')]
        params: Vec<f64>,
    },
    /// initialize particles state
    Initialize {
        /// type of crystal cell
        #[arg(short = 't', long, value_enum)]
        crystal_cell_type: CrystalCellType,
        /// size of this cell (unit cells count, x y z)
        #[arg(short = 's', long, num_args = 3, value_delimiter = ' ')]
        size: Vec<u32>,
        /// name of particle to initialize
        #[arg(short = 'n', long)]
        particle_name: String,
        /// mass of particle to initialize (10^-27 kg)
        #[arg(short = 'm', long)]
        particle_mass: f64,
        /// radius of particle to initialize (nm)
        #[arg(short = 'r', long)]
        particle_radius: f64,
        /// lattice cell (nm)
        #[arg(short = 'l', long)]
        lattice_cell: f64,
        /// temperature (K)
        #[arg(short = 'T', long)]
        temperature: f64,
    },
    /// run solver on particle state
    Solve {
        #[arg(long)]
        threads_count: Option<usize>,
        #[arg(short = 's', long)]
        state_number: usize,
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
        /// Use file with potentials for any id pair (potential.json).
        /// If not it uses default potential for Argon
        #[arg(short = 'p', long)]
        use_potentials: bool,
        /// how much iterations to count
        #[arg(short = 'c', long)]
        iteration_count: usize,
        /// how long each iteration should take
        #[arg(short = 't', long)]
        delta_time: f64,
    },
    /// calculate macro parameters for solved state
    SolveMacroParameters {
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
        /// Use file with potentials for any id pair (potential.json).
        /// If not it uses default potential for Argon
        #[arg(long)]
        use_potentials: bool,
    },
    /// Prints impulse (momentum) on first and last step
    CheckImpulse,
    /// Prints particle count in simulation
    ParticleCount,
    /// Outputs to file data for velocities histogram
    GenerateVelocitiesHistogram {
        /// Step on which histogram will be created
        #[arg(short = 's', long)]
        state_number: usize,
        #[arg(long, num_args = 1..65536, value_delimiter = ' ')]
        particle_types: Vec<u16>,
    },
}
