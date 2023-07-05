use std::path::PathBuf;
use clap::{Parser, Subcommand};
use clap::ValueEnum;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// path to file with particles data
    #[arg(short, long)]
    file: PathBuf,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, ValueEnum)]
enum CrystalCellType {
    /// uniform grid like in gases
    Uniform,
}

#[derive(Subcommand)]
enum Commands {
    /// initialize particles state
    Initialize {
        /// path to file
        #[arg(short, long)]
        crystal_cell_type: CrystalCellType,
        /// size of this cell
        #[arg(short, long, num_args = 3, value_delimiter = ' ')]
        size: Vec<f64>,
    },

}

pub fn get_args() -> Args {
    let args = Args::parse();
    match &args.command {
        Commands::Initialize {
            crystal_cell_type, size
        } => {

        }
    }
    args
}