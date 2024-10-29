use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author="John White", version, about="Merge/Split WAV,FLAC files and create CUE sheet", long_about = None, arg_required_else_help = true)]
pub struct Cli {
    /// Force overwriting of output files
    #[arg(long, short, default_value = "false")]
    pub force: bool,
    /// Silent mode
    #[arg(long, short, default_value = "false")]
    pub silent: bool,
    /// Do not print anything of any kind, including warnings or errors
    #[arg(long, short, default_value = "false")]
    pub totally_silent: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Merge WAV,FLAC files into single WAV file and create CUE sheet
    Merge {
        /// Create CUE sheet
        #[arg(long, short, default_value = "false")]
        cue: bool,
        /// Set CUE album name
        #[arg(long, short)]
        title: Option<String>,
        /// Set CUE artist
        #[arg(long, short)]
        performer: Option<String>,
        /// Add REM comments to CUE
        #[arg(long, short)]
        rem: Option<Vec<String>>,
        /// Make sure input files samples matches output file samples
        #[arg(long, short, default_value = "false")]
        verify: bool,
        /// Input files
        #[arg(long, short, required = true, value_delimiter = ',', num_args = 1..)]
        input: Vec<PathBuf>,
        /// Output WAV file
        #[arg(long, short, required = true)]
        output: PathBuf,
    },
    /// Split WAV,FLAC file into separate tracks using CUE sheet
    Split {
        /// Create multiple file CUE sheet
        #[arg(long, short, default_value = "false")]
        cue: bool,
        /// Path to input CUE sheet
        #[arg(long, short, required = true)]
        input: PathBuf,
        /// Output directory for splitted tracks
        #[arg(long, short)]
        output_dir: Option<PathBuf>,
        /// Make sure output files samples matches input file samples
        #[arg(long, short, default_value = "false")]
        verify: bool,
        /// File name format for splitted tracks
        #[arg(long, short)]
        format: Option<String>,
    },
    /// Print examples
    Examples {},
}
