use std::{
    fs::{self, File},
    io::BufWriter,
    path::PathBuf,
};

use anyhow::{bail, Context};
use blake3::Hasher;
use claxon::FlacReader;
use hound::{Sample, SampleFormat, WavReader, WavSpec, WavWriter};
use num_traits::ToBytes;

use crate::{
    cli::Cli,
    utils::{create_sample_progress, promt_overwrite},
};

pub struct Duration {
    pub file: PathBuf,
    pub duration_seconds: f64,
    pub duration_samples: u64,
}

pub struct AudioInfo {
    pub spec: WavSpec,
    pub total_samples: u64,
}

pub fn wav_split_samples<S>(
    input_file: &PathBuf,
    audio_spec: WavSpec,
    durations: &Vec<Duration>,
    cli: &Cli,
) -> Result<Vec<PathBuf>, anyhow::Error>
where
    S: Sample,
{
    let mut reader = WavReader::open(input_file)?;
    let mut samples = reader.samples::<S>();
    let mut output_wavs: Vec<PathBuf> = vec![];

    for duration in durations {
        if !cli.force && fs::exists(&duration.file).context("Can't check existence of file")? {
            if !promt_overwrite(&duration.file).context("Promt failed")? {
                continue;
            }
        }

        let mut output_wav = WavWriter::create(&duration.file, audio_spec)
            .context("Failed to create output WAV file")?;

        if !cli.silent {
            println!(
                "Writing {} ...",
                &duration
                    .file
                    .file_name()
                    .context("Failed to get file name")?
                    .to_str()
                    .context("to_str failed")?
            );
        }

        let pb = create_sample_progress(duration.duration_samples, cli)?;

        for _ in 0..duration.duration_samples as u64 {
            output_wav.write_sample(samples.next().context("Failed to get next sample")??)?;

            match pb {
                Some(ref v) => v.inc(1),
                None => (),
            }
        }

        output_wav
            .finalize()
            .context("Failed to update the WAVE header")?;

        output_wavs.push(duration.file.clone());
    }

    Ok(output_wavs)
}

pub fn flac_split_samples(
    input_file: &PathBuf,
    audio_spec: WavSpec,
    durations: &Vec<Duration>,
    cli: &Cli,
) -> Result<Vec<PathBuf>, anyhow::Error> {
    let mut reader = FlacReader::open(input_file)?;
    let mut samples = reader.samples();
    let mut output_wavs: Vec<PathBuf> = vec![];

    for duration in durations {
        if !cli.force && fs::exists(&duration.file).context("Can't check existence of file")? {
            if !promt_overwrite(&duration.file).context("Promt failed")? {
                continue;
            }
        }

        let mut output_wav = WavWriter::create(&duration.file, audio_spec)
            .context("Failed to create output WAV file")?;

        if !cli.silent {
            println!(
                "Writing {} ...",
                &duration
                    .file
                    .file_name()
                    .context("Failed to get file name")?
                    .to_str()
                    .context("to_str failed")?
            );
        }

        let pb = create_sample_progress(duration.duration_samples, &cli)?;

        for _ in 0..duration.duration_samples as u64 {
            output_wav.write_sample(samples.next().context("Failed to get next sample")??)?;

            match pb {
                Some(ref v) => v.inc(1),
                None => (),
            }
        }

        output_wav
            .finalize()
            .context("Failed to update the WAVE header")?;

        output_wavs.push(duration.file.clone());
    }

    Ok(output_wavs)
}

pub fn wav_copy_samples<S>(
    from_file: &PathBuf,
    to_file: &mut WavWriter<BufWriter<File>>,
    cli: &Cli,
) -> Result<u64, anyhow::Error>
where
    S: Sample,
{
    let mut reader = WavReader::open(from_file)?;
    let mut samples_written: u64 = 0;
    let pb = create_sample_progress(reader.len().into(), cli)?;
    let samples = reader.samples::<S>();

    for sample in samples {
        to_file.write_sample(sample?)?;
        samples_written += 1;

        match pb {
            Some(ref v) => v.set_position(samples_written),
            None => (),
        }
    }

    Ok(reader.duration() as u64)
}

pub fn flac_copy_samples(
    from_file: &PathBuf,
    to_file: &mut WavWriter<BufWriter<File>>,
    cli: &Cli,
) -> Result<u64, anyhow::Error> {
    let mut reader = FlacReader::open(from_file)?;
    let samples_count = reader
        .streaminfo()
        .samples
        .context("Failed to get samples total number")?
        * reader.streaminfo().channels as u64;
    let mut samples_written: u64 = 0;
    let pb = create_sample_progress(samples_count, cli)?;
    let samples = reader.samples();

    for sample in samples {
        to_file.write_sample(sample?)?;
        samples_written += 1;

        match pb {
            Some(ref v) => v.set_position(samples_written),
            None => (),
        }
    }

    Ok(reader
        .streaminfo()
        .samples
        .context("Failed to get samples total number")?)
}

pub fn wav_hash_samples<S>(
    from_file: &PathBuf,
    hasher: &mut Hasher,
    cli: &Cli,
) -> Result<(), anyhow::Error>
where
    S: Sample + ToBytes,
{
    let mut reader = WavReader::open(from_file)?;
    let mut samples_readed: u64 = 0;
    let pb = create_sample_progress(reader.len().into(), cli)?;

    let mut bytes;
    let samples = reader.samples::<S>();
    for sample in samples {
        bytes = sample?.to_be_bytes();
        hasher.update(bytes.as_ref());
        samples_readed += 1;

        match pb {
            Some(ref v) => v.set_position(samples_readed),
            None => (),
        }
    }

    Ok(())
}

pub fn flac_hash_samples(
    from_file: &PathBuf,
    hasher: &mut Hasher,
    cli: &Cli,
) -> Result<(), anyhow::Error> {
    let mut reader = FlacReader::open(from_file)?;
    let samples_count = reader
        .streaminfo()
        .samples
        .context("Failed to get samples total number")?
        * reader.streaminfo().channels as u64;
    let mut samples_readed: u64 = 0;
    let pb = create_sample_progress(samples_count, cli)?;

    let mut bytes;
    let samples = reader.samples();
    for sample in samples {
        bytes = sample?.to_be_bytes();
        hasher.update(bytes.as_ref());
        samples_readed += 1;

        match pb {
            Some(ref v) => v.set_position(samples_readed),
            None => (),
        }
    }

    Ok(())
}

pub fn verify_samples(
    sample_format: SampleFormat,
    input: &Vec<PathBuf>,
    output: &PathBuf,
    cli: &Cli,
) -> Result<(), anyhow::Error> {
    if !cli.silent {
        println!("Verifying ...");
    }

    let mut input_hasher = Hasher::new();
    for file in input {
        if !cli.silent {
            println!(
                "Reading {} ...",
                &file
                    .file_name()
                    .context("Failed to get file name")?
                    .to_str()
                    .context("to_str failed")?
            );
        }

        hash_samples(file, &sample_format, &mut input_hasher, cli)?;
    }

    let input_hash = input_hasher.finalize();

    if !cli.silent {
        println!(
            "Reading {} ...",
            &output
                .file_name()
                .context("Failed to get file name")?
                .to_str()
                .context("to_str failed")?
        );
    }

    let mut output_hasher = Hasher::new();
    hash_samples(output, &sample_format, &mut output_hasher, cli)?;

    let output_hash = output_hasher.finalize();

    if input_hash != output_hash {
        bail!("Verify FAILED: Samples mismatch");
    } else {
        if !cli.totally_silent {
            println!("Verify OK");
        }
    }

    Ok(())
}

pub fn hash_samples(
    file: &PathBuf,
    sample_format: &SampleFormat,
    hasher: &mut Hasher,
    cli: &Cli,
) -> Result<(), anyhow::Error> {
    match file
        .extension()
        .context("Failed to get file extension")?
        .to_str()
        .context("to_str failed")?
        .to_lowercase()
        .as_ref()
    {
        "wav" => match sample_format {
            SampleFormat::Float => {
                wav_hash_samples::<f32>(file, hasher, cli).context("Failed to hash samples")?
            }
            SampleFormat::Int => {
                wav_hash_samples::<i32>(file, hasher, cli).context("Failed to hash samples")?
            }
        },
        "flac" => flac_hash_samples(file, hasher, cli).context("Failed to hash samples")?,
        _ => bail!("Unsupported format"),
    };

    Ok(())
}

pub fn get_wav_info(file: &PathBuf) -> Result<AudioInfo, anyhow::Error> {
    let reader = WavReader::open(file)?;
    Ok(AudioInfo {
        spec: reader.spec(),
        total_samples: reader.len() as u64,
    })
}

pub fn get_flac_info(file: &PathBuf) -> Result<AudioInfo, anyhow::Error> {
    let reader = FlacReader::open(file)?;
    let spec = WavSpec {
        channels: reader.streaminfo().channels as u16,
        sample_rate: reader.streaminfo().sample_rate,
        bits_per_sample: reader.streaminfo().bits_per_sample as u16,
        sample_format: SampleFormat::Int,
    };

    Ok(AudioInfo {
        spec,
        total_samples: reader
            .streaminfo()
            .samples
            .context("Failed to get FLAC total samples")?
            * spec.channels as u64,
    })
}

pub fn get_audio_info(file: &PathBuf) -> Result<AudioInfo, anyhow::Error> {
    match file
        .extension()
        .context("Failed to get file extension")?
        .to_str()
        .context("to_str failed")?
        .to_lowercase()
        .as_ref()
    {
        "wav" => Ok(get_wav_info(&file).context("Failed to get information about the WAVE file")?),
        "flac" => {
            Ok(get_flac_info(&file).context("Failed to get information about the FLAC file")?)
        }
        _ => bail!("Unsupported format"),
    }
}
