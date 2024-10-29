use std::{fs, path::PathBuf};

use anyhow::{bail, Context};
use cue_rw::{CUEFile, CUETimeStamp, CUETrack};

use crate::{audio::Duration, cli::Cli, track_msf::TrackMSF, utils::promt_overwrite};

pub fn merge_create_cue(
    title: &Option<String>,
    performer: &Option<String>,
    rem: &Option<Vec<String>>,
    output: &PathBuf,
    durations: &Vec<Duration>,
    cli: &Cli,
) -> Result<(), anyhow::Error> {
    if !cli.silent && !cli.totally_silent {
        println!("Creating CUE file ...");
    }

    let mut cue = CUEFile::new();

    cue.title = title.clone().unwrap_or("Album".to_string());
    cue.performer = performer.clone().unwrap_or("Artist".to_string());

    match rem {
        Some(comments) => {
            comments.iter().for_each(|c| {
                cue.comments.push(c.clone());
            });
        }
        None => (),
    }

    cue.files.push(
        output
            .file_name()
            .context("Failed to get file name")?
            .to_str()
            .context("to_str failed")?
            .to_string(),
    );

    let mut output_cue: PathBuf = PathBuf::new();

    output_cue.push(
        output
            .parent()
            .context("Failed to get output file parent dir")?,
    );

    output_cue.push(format!(
        "{}.cue",
        output
            .file_stem()
            .context("Failed to get file name")?
            .to_str()
            .context("to_str failed")?
            .to_string()
    ));

    let mut last_duration: f64 = 0.0;
    for duration in durations {
        let mut track = CUETrack::new();
        let track_duration = TrackMSF::new(last_duration).to_string();
        track.title = duration
            .file
            .file_stem()
            .context("Failed to get file name")?
            .to_str()
            .context("to_str failed")?
            .to_string();
        track.performer = Some(cue.performer.clone());
        track.indices.push((
            1,
            CUETimeStamp::try_from(track_duration.as_ref())
                .context("Failed to convert TrackMSF to CUETimeStamp")?,
        ));
        track
            .comments
            .push(format!("DURATION {}", duration.duration_samples));
        cue.tracks.push((0, track));
        last_duration += duration.duration_seconds;
    }

    if !cli.force && fs::exists(&output_cue).context("Can't check existence of file")? {
        if !promt_overwrite(&output_cue).context("Promt failed")? {
            return Ok(());
        }
    }

    fs::write(&output_cue, cue.to_string()).context("Failed to write CUE file")?;

    Ok(())
}

pub fn split_create_cue(
    cue_file: &CUEFile,
    input: &PathBuf,
    durations: &Vec<Duration>,
    cli: &Cli,
) -> Result<(), anyhow::Error> {
    if !cli.silent && !cli.totally_silent {
        println!("Creating CUE file ...");
    }

    let mut cue_multiple = CUEFile::new();

    cue_multiple.title = cue_file.title.clone();
    cue_multiple.performer = cue_file.performer.clone();
    cue_multiple.comments = cue_file.comments.clone();

    let mut output_cue: PathBuf = PathBuf::new();

    output_cue.push(
        input
            .parent()
            .context("Failed to get input file parent dir")?,
    );

    output_cue.push(format!(
        "{}_multiple.cue",
        input
            .file_stem()
            .context("Failed to get file name")?
            .to_str()
            .context("to_str failed")?
            .to_string()
    ));

    let mut cue_file_tracks = cue_file.tracks.iter();

    for (i, duration) in durations.iter().enumerate() {
        let cue_file_next_track = cue_file_tracks
            .next()
            .context("Failed to get input cue next track")?;

        cue_multiple.files.push(
            duration
                .file
                .file_name()
                .context("Failed to get file name")?
                .to_str()
                .context("to_str failed")?
                .to_string(),
        );

        let mut track = CUETrack::new();
        track.title = cue_file_next_track.1.title.clone();
        track.performer = cue_file_next_track.1.performer.clone();
        track.indices.push((1, CUETimeStamp::new(0, 0, 0)));
        track.comments = cue_file_next_track.1.comments.clone();
        cue_multiple.tracks.push((i, track));
    }

    if !cli.force && fs::exists(&output_cue).context("Can't check existence of file")? {
        if !promt_overwrite(&output_cue).context("Promt failed")? {
            return Ok(());
        }
    }

    fs::write(&output_cue, cue_multiple.to_string()).context("Failed to write CUE file")?;

    Ok(())
}

pub fn cue_msf_to_samples(
    indices: &Vec<(u8, CUETimeStamp)>,
    sample_rate: u32,
) -> Result<u64, anyhow::Error> {
    let cue_ts = indices
        .iter()
        .find_map(|t| if t.0 == 1 { Some(t) } else { None });

    match cue_ts {
        Some(ts) => {
            let duration = TryInto::<TrackMSF>::try_into(ts.1.to_string().as_ref())?;
            Ok((duration.to_duration_seconds() * sample_rate as f64) as u64)
        }
        None => bail!("Can`t find track INDEX 01"),
    }
}
