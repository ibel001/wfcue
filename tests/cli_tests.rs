use std::{fs, path::PathBuf};

use duct::cmd;
use duct_sh::sh_dangerous;
use hound::{SampleFormat, WavSpec, WavWriter};
use serial_test::serial;

fn wfcue_exe() -> PathBuf {
    env!("CARGO_BIN_EXE_wfcue").into()
}

fn get_test_dir() -> PathBuf {
    wfcue_exe().parent().unwrap().to_path_buf()
}

#[test]
#[serial]
fn test_sh_merge_wavs_wildcard_spaces() {
    remove_tmp_files(true);
    create_test_wavs(true);

    let test_dir = get_test_dir();
    let exe = wfcue_exe().to_str().unwrap().to_string();
    let cmd = format!(
        r#"{exe} --silent merge --cue --title Album --performer Artist --verify --input *.wav --output output.wav"#
    );

    sh_dangerous(&cmd).dir(test_dir).run().unwrap();
    remove_tmp_files(true);
}

#[test]
#[serial]
fn test_sh_merge_wavs_wildcard() {
    remove_tmp_files(false);
    create_test_wavs(false);

    let test_dir = get_test_dir();
    let exe = wfcue_exe().to_str().unwrap().to_string();
    let cmd = format!(
        r#"{exe} --silent merge --cue --title Album --performer Artist --verify --input *.wav --output output.wav"#
    );

    sh_dangerous(&cmd).dir(test_dir).run().unwrap();
    remove_tmp_files(false);
}

#[test]
#[serial]
fn test_cmd_merge_wavs_multiple_spaces() {
    remove_tmp_files(true);
    create_test_wavs(true);

    let test_dir = get_test_dir();
    cmd!(
        wfcue_exe(),
        "--silent",
        "merge",
        "--cue",
        "--title",
        r#""Album""#,
        "--performer",
        "Artist",
        "--rem",
        r#""COMMENT wfcue""#,
        "--verify",
        "--input",
        "1 1 1.wav,2 2 2.wav,3 3 3.wav",
        "--output",
        "output.wav"
    )
    .dir(test_dir)
    .run()
    .unwrap();

    remove_tmp_files(true);
}

#[test]
#[serial]
fn test_cmd_merge_wavs_multiple() {
    remove_tmp_files(false);
    create_test_wavs(false);

    let test_dir = get_test_dir();
    cmd!(
        wfcue_exe(),
        "--silent",
        "merge",
        "--cue",
        "--title",
        r#""Album""#,
        "--performer",
        "Artist",
        "--rem",
        r#""COMMENT wfcue""#,
        "--verify",
        "--input",
        "1.wav,2.wav,3.wav",
        "--output",
        "output.wav"
    )
    .dir(test_dir)
    .run()
    .unwrap();

    remove_tmp_files(false);
}

#[test]
#[serial]
fn test_sh_split_wav() {
    remove_tmp_files(false);
    create_test_wavs(false);

    let test_dir = get_test_dir();
    cmd!(
        wfcue_exe(),
        "--silent",
        "merge",
        "--cue",
        "--title",
        r#""Album""#,
        "--performer",
        "Artist",
        "--rem",
        r#""COMMENT wfcue""#,
        "--verify",
        "--input",
        "1.wav,2.wav,3.wav",
        "--output",
        "output.wav"
    )
    .dir(&test_dir)
    .run()
    .unwrap();

    let exe = wfcue_exe().to_str().unwrap().to_string();
    let cmd = format!("{exe} --silent split --input output.cue --verify --format %track%");

    sh_dangerous(&cmd).dir(test_dir).run().unwrap();
    remove_tmp_files(false);
}

fn create_test_wavs(spaces: bool) -> Vec<PathBuf> {
    let test_dir = get_test_dir();

    let spec = WavSpec {
        channels: 2,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut input: Vec<PathBuf> = vec![];

    for i in 1..=3 {
        let mut test_file = test_dir.clone();

        if spaces {
            test_file.push(format!("{i} {i} {i}.wav"));
        } else {
            test_file.push(format!("{i}.wav"));
        }

        let mut writer = WavWriter::create(&test_file, spec).unwrap();

        for t in (0..88200).map(|x| x as f32 / 44100.0) {
            let sample = (t * 440.0 + i as f32 * 2.0 * std::f32::consts::PI).sin();
            let amplitude = i16::MAX as f32;
            writer.write_sample((sample * amplitude) as i16).unwrap();
        }

        writer.finalize().unwrap();
        input.push(test_file);
    }

    input
}

fn remove_tmp_files(spaces: bool) {
    let test_dir = get_test_dir();

    let mut output = test_dir.clone();
    output.push("output.wav");

    let mut output_cue = test_dir.clone();
    output_cue.push("output.cue");

    for i in 1..=3 {
        let mut test_file = test_dir.clone();

        if spaces {
            test_file.push(format!("{i} {i} {i}.wav"));
        } else {
            test_file.push(format!("{i}.wav"));
        }

        if fs::exists(&test_file).unwrap() {
            fs::remove_file(test_file).unwrap();
        }

        let mut test_file = test_dir.clone();
        if spaces {
            test_file.push(format!("{:02} {:02} {:02}.wav", i, i, i));
        } else {
            test_file.push(format!("{:02}.wav", i));
        }

        if fs::exists(&test_file).unwrap() {
            fs::remove_file(test_file).unwrap();
        }
    }

    if fs::exists(&output).unwrap() {
        fs::remove_file(output).unwrap();
    }

    if fs::exists(&output_cue).unwrap() {
        fs::remove_file(output_cue).unwrap();
    }
}
