use std::{fs, path::PathBuf};

use anyhow::{bail, Context};
use cue_rw::CUEFile;
use hound::{SampleFormat, WavWriter};

use crate::{
    audio::{
        flac_copy_samples, flac_split_samples, get_audio_info, verify_samples, wav_copy_samples,
        wav_split_samples, Duration,
    },
    cli::Cli,
    cue::{cue_msf_to_samples, merge_create_cue, split_create_cue},
    utils::{promt_output_in_input, promt_overwrite},
};

pub fn merge(
    cue: bool,
    title: &Option<String>,
    performer: &Option<String>,
    rem: &Option<Vec<String>>,
    verify: bool,
    input: &Vec<PathBuf>,
    output: &PathBuf,
    cli: &Cli,
) -> Result<Vec<PathBuf>, anyhow::Error> {
    if !cli.totally_silent && input.contains(output) {
        if !promt_output_in_input(output).context("Promt failed")? {
            return Ok(vec![]);
        }
    }

    if !cli.silent {
        println!(
            "Reading properties of the audio data from {}",
            &input[0]
                .file_name()
                .context("Failed to get file name")?
                .to_str()
                .context("to_str failed")?
        );
    }

    if !cli.silent {
        println!(
            "Output file {}",
            output
                .file_name()
                .context("Failed to get file name")?
                .to_str()
                .context("to_str failed")?
        );
    }

    let audio_info = get_audio_info(&input[0])?;
    let mut durations: Vec<Duration> = vec![];

    if !cli.force && fs::exists(output).context("Can't check existence of file")? {
        if !promt_overwrite(output).context("Promt failed")? {
            return Ok(vec![]);
        }
    }

    let mut output_wav =
        WavWriter::create(output, audio_info.spec).context("Failed to create output WAV file")?;

    for file in input.iter() {
        if !cli.silent {
            println!(
                "Merging {} ...",
                &file
                    .file_name()
                    .context("Failed to get file name")?
                    .to_str()
                    .context("to_str failed")?
            );
        }

        let duration_samples = match file
            .extension()
            .context("Failed to get file extension")?
            .to_str()
            .context("to_str failed")?
            .to_lowercase()
            .as_ref()
        {
            "wav" => match audio_info.spec.sample_format {
                SampleFormat::Float => wav_copy_samples::<f32>(&file, &mut output_wav, cli)
                    .context("Failed to copy samples")?,
                SampleFormat::Int => wav_copy_samples::<i32>(&file, &mut output_wav, cli)
                    .context("Failed to copy samples")?,
            },
            "flac" => {
                flac_copy_samples(&file, &mut output_wav, cli).context("Failed to copy samples")?
            }
            _ => bail!("Unsupported format"),
        };

        let duration_seconds = duration_samples as f64 / audio_info.spec.sample_rate as f64;
        let duration = Duration {
            file: file.clone(),
            duration_seconds,
            duration_samples,
        };

        durations.push(duration);
    }

    output_wav
        .finalize()
        .context("Failed to update the WAVE header")?;

    if verify {
        verify_samples(audio_info.spec.sample_format, &input, output, cli)?;
    }

    if cue {
        merge_create_cue(title, performer, &rem, output, &durations, cli)?;
    }

    Ok(vec![output.clone()])
}

pub fn split(
    cue: bool,
    input: &PathBuf,
    output_dir: &Option<PathBuf>,
    verify: bool,
    format: &Option<String>,
    cli: &Cli,
) -> Result<Vec<PathBuf>, anyhow::Error> {
    let cue_text = fs::read_to_string(input).context("Failed to read CUE file")?;
    let cue_file = CUEFile::try_from(cue_text.as_ref()).context("Failed to parse CUE sheet")?;

    let mut audio_file = input
        .parent()
        .context("Failed to get parent dir")?
        .to_path_buf();

    audio_file.push(&cue_file.files[0]);

    if !cli.silent {
        println!(
            "Reading properties of the audio data from {}",
            &audio_file
                .file_name()
                .context("Failed to get file name")?
                .to_str()
                .context("to_str failed")?
        );
    }

    let audio_info = get_audio_info(&audio_file)?;
    let mut durations: Vec<Duration> = vec![];

    if !cli.silent {
        println!("Reading track info from CUE file ...");
    }

    let mut cue_tracks_iter = cue_file.tracks.iter().peekable();
    let mut track_num = 0;

    loop {
        let Some(track) = cue_tracks_iter.next() else {
            break;
        };

        let track = &track.1;
        track_num += 1;

        let rem_duration = track.comments.iter().find_map(|s| {
            if s.starts_with("DURATION ") {
                Some(s)
            } else {
                None
            }
        });

        let duration = match rem_duration {
            Some(rem) => {
                let split: Vec<&str> = rem.split(" ").collect();
                if split.len() < 2 {
                    bail!("Failed to parse REM DURATION")
                }
                let samples = split[1]
                    .parse::<u64>()
                    .context("Failed to parse REM DURATION")?;
                samples * audio_info.spec.channels as u64
            }
            None => {
                // Fallback to MSF
                let peek_track = cue_tracks_iter.peek();
                let track_pos = cue_msf_to_samples(&track.indices, audio_info.spec.sample_rate)?;

                match peek_track {
                    Some(next_track) => {
                        let next_track_pos =
                            cue_msf_to_samples(&next_track.1.indices, audio_info.spec.sample_rate)?;
                        let samples = next_track_pos - track_pos;
                        samples * audio_info.spec.channels as u64
                    }
                    None => {
                        // Last track
                        let mut samples: u64 =
                            cue_msf_to_samples(&track.indices, audio_info.spec.sample_rate)?;
                        samples =
                            (audio_info.total_samples / audio_info.spec.channels as u64) - samples;
                        samples * audio_info.spec.channels as u64
                    }
                }
            }
        };

        let mut output_file = PathBuf::new();

        match output_dir {
            Some(d) => output_file.push(d),
            None => {
                output_file.push(
                    input
                        .parent()
                        .context("Failed to get input file parent dir")?,
                );
            }
        }

        let output_filename = match format {
            Some(f) => {
                f.replace("%track%", &format!("{:02}", track_num))
                    .replace(
                        "%artist%",
                        &track.performer.clone().context("Failed to get performer")?,
                    )
                    .replace("%title%", &track.title)
                    + ".wav"
            }
            None => format!(
                "{:02} {} - {}.wav",
                track_num,
                track.performer.clone().unwrap_or("Artist".to_string()),
                track.title
            ),
        };

        output_file.push(output_filename);

        durations.push(Duration {
            file: output_file,
            duration_samples: duration,
            duration_seconds: duration as f64 / audio_info.spec.sample_rate as f64,
        });
    }

    let output_wavs = match audio_file
        .extension()
        .context("Failed to get file extension")?
        .to_str()
        .context("to_str failed")?
        .to_lowercase()
        .as_ref()
    {
        "wav" => match audio_info.spec.sample_format {
            SampleFormat::Float => {
                wav_split_samples::<f32>(&audio_file, audio_info.spec, &durations, &cli)
                    .context("Failed to copy samples")?
            }
            SampleFormat::Int => {
                wav_split_samples::<i32>(&audio_file, audio_info.spec, &durations, &cli)
                    .context("Failed to copy samples")?
            }
        },
        "flac" => flac_split_samples(&audio_file, audio_info.spec, &durations, &cli)
            .context("Failed to copy samples")?,
        _ => bail!("Unsupported format"),
    };

    if verify {
        verify_samples(
            audio_info.spec.sample_format,
            &output_wavs,
            &audio_file,
            cli,
        )?;
    }

    if cue {
        split_create_cue(&cue_file, input, &durations, cli)?;
    }

    Ok(output_wavs)
}

pub fn examples() -> Vec<PathBuf> {
    let text = r#"Merge all wav files in the current working directory and create CUE sheet:

wfcue merge --cue --title "Album" --performer "Artist" --rem "COMMENT wfcue" --verify --input *.wav --output "Artist - Album.wav"

Merge selected wav files in the current working directory and create CUE sheet:

wfcue merge --cue --title "Album" --performer "Artist" --rem "COMMENT wfcue" --verify --input 1.wav,2.wav,3.wav --output "Artist - Album.wav"

Split a single large audio file containing the entire album into the separate audio tracks:

wfcue split --input "Artist - Album.cue" --verify --format "%track%. %artist% - %title%"

Split a single large audio file containing the entire album into the separate audio tracks and create multiple file CUE sheet:

wfcue split --cue --input "Artist - Album.cue" --verify --format "%track%. %artist% - %title%"

Merge all wav files in the current working directory and create CUE sheet also overwrite existing files and use silent mode:

wfcue --force --silent merge --cue --title "Album" --performer "Artist" --rem "COMMENT wfcue" --rem "COMPOSER test" --verify --input *.wav --output "Artist - Album.wav""#;
    println!("{}", text);
    vec![]
}
