use anyhow::bail;
use clap::Parser;
use cli::{Cli, Commands};
use commands::{examples, merge, split};

mod audio;
mod cli;
mod commands;
mod cue;
mod tests;
mod track_msf;
mod utils;

fn main() -> Result<(), anyhow::Error> {
    let mut cli = Cli::parse_from(wild::args());

    if cli.totally_silent {
        cli.silent = true;
    }

    match process_command(&cli) {
        Ok(_) => {
            if !&cli.totally_silent {
                println!("Done.")
            }
        }
        Err(e) => {
            if !&cli.totally_silent {
                bail!(e);
            }
        }
    };

    Ok(())
}

fn process_command(cli: &Cli) -> Result<(), anyhow::Error> {
    match &cli.command {
        Commands::Merge {
            cue,
            title,
            performer,
            rem,
            verify,
            input,
            output,
        } => merge(*cue, &title, &performer, rem, *verify, input, output, &cli)?,
        Commands::Split {
            cue,
            input,
            output_dir,
            verify,
            format,
        } => split(*cue, input, output_dir, *verify, format, &cli)?,
        Commands::Examples {} => examples(),
    };

    Ok(())
}
