use std::{fmt::Write, path::PathBuf};

use anyhow::{bail, Context};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use inquire::Confirm;

use crate::cli::Cli;

pub fn create_sample_progress(len: u64, cli: &Cli) -> Result<Option<ProgressBar>, anyhow::Error> {
    if cli.silent || cli.totally_silent {
        return Ok(None);
    }

    let pb = ProgressBar::new(len);

    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} samples ({eta})",
        )
        ?
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap_or_default()
        })
        .progress_chars("#>-"),
    );

    Ok(Some(pb))
}

pub fn promt_output_in_input(file: &PathBuf) -> Result<bool, anyhow::Error> {
    let ans = Confirm::new(&format!(
        "Input file list contains {} output file, are you sure you want to continue?",
        file.file_name()
            .context("Failed to get file name")?
            .to_str()
            .context("to_str failed")?
    ))
    .with_default(false)
    .prompt();

    match ans {
        Ok(v) => Ok(v),
        Err(e) => bail!(e),
    }
}

pub fn promt_overwrite(file: &PathBuf) -> Result<bool, anyhow::Error> {
    let ans = Confirm::new(&format!(
        "File {} already exists, overwrite?",
        file.file_name()
            .context("Failed to get file name")?
            .to_str()
            .context("to_str failed")?
    ))
    .with_default(false)
    .prompt();

    match ans {
        Ok(v) => Ok(v),
        Err(e) => bail!(e),
    }
}
