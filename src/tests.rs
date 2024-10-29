#[cfg(test)]
mod commands {
    use std::{
        fs::{self, File},
        path::PathBuf,
        process::Command,
    };

    use blake3::Hash;
    use hound::{SampleFormat, WavSpec, WavWriter};
    use serial_test::serial;

    use crate::*;

    #[test]
    #[serial]
    fn test_wav_split_msf() {
        test_split(false, true);
    }

    #[test]
    #[serial]
    fn test_flac_split_msf() {
        test_split(true, true);
    }

    #[test]
    #[serial]
    fn test_wav_split() {
        test_split(false, false);
    }

    #[test]
    #[serial]
    fn test_flac_split() {
        test_split(true, false);
    }

    #[test]
    #[serial]
    fn test_wav_merge() {
        test_merge(false, true);
    }

    #[test]
    #[serial]
    fn test_flac_merge() {
        test_merge(true, true);
    }

    fn test_merge(flac: bool, remove_test_files: bool) -> Vec<PathBuf> {
        remove_tmp_files(flac);
        let test_dir = get_test_dir();

        let mut output = test_dir.clone();
        output.push("output.wav");

        let mut output_cue = test_dir.clone();
        output_cue.push("output.cue");

        let cli = Cli {
            force: false,
            silent: false,
            totally_silent: false,
            command: cli::Commands::Merge {
                cue: true,
                title: Some("Album".to_string()),
                performer: Some("Artist".to_string()),
                rem: None,
                verify: true,
                input: vec![],
                output: PathBuf::new(),
            },
        };

        let input = create_test_wavs(flac);

        merge(
            true,
            &Some("Album".to_string()),
            &Some("Artist".to_string()),
            &Some(vec![r#"COMPOSER "TEST""#.to_string()]),
            true,
            &input,
            &output,
            &cli,
        )
        .unwrap();

        let test_cue = r#"REM COMPOSER "TEST"
TITLE "Album"
PERFORMER "Artist"
FILE "output.wav" WAVE
  TRACK 01 AUDIO
    TITLE "1"
    PERFORMER "Artist"
    INDEX 01 00:00:00
    REM DURATION 44100
  TRACK 02 AUDIO
    TITLE "2"
    PERFORMER "Artist"
    INDEX 01 00:01:00
    REM DURATION 44100
  TRACK 03 AUDIO
    TITLE "3"
    PERFORMER "Artist"
    INDEX 01 00:02:00
    REM DURATION 44100"#;

        assert!(fs::read_to_string(output_cue).unwrap() == test_cue);

        if remove_test_files {
            remove_tmp_files(flac);
            return vec![];
        }

        input
    }

    fn test_split(flac: bool, test_msf: bool) {
        let test_dir = get_test_dir();

        let mut output = test_dir.clone();
        output.push("output.wav");

        let mut output_cue = test_dir.clone();
        output_cue.push("output.cue");

        let mut output_cue_multiple = test_dir.clone();
        output_cue_multiple.push("output_multiple.cue");

        let input = test_merge(flac, false);

        if flac {
            encode_to_flac(&output);
            output.pop();
            output.push("output.flac");
            let cue_file = fs::read_to_string(&output_cue)
                .unwrap()
                .replace("output.wav", "output.flac");
            fs::write(&output_cue, cue_file).unwrap();
        }

        let cli = Cli {
            force: false,
            silent: false,
            totally_silent: false,
            command: Commands::Split {
                cue: true,
                input: output_cue.clone(),
                output_dir: Some(test_dir.clone()),
                verify: true,
                format: None,
            },
        };

        if test_msf {
            let test_cue_msf = format!(
                r#"REM COMPOSER "TEST"
TITLE "Album"
PERFORMER "Artist"
FILE "{}" WAVE
  TRACK 01 AUDIO
    TITLE "1"
    PERFORMER "Artist"
    INDEX 01 00:00:00
  TRACK 02 AUDIO
    TITLE "2"
    PERFORMER "Artist"
    INDEX 01 00:01:00
  TRACK 03 AUDIO
    TITLE "3"
    PERFORMER "Artist"
    INDEX 01 00:02:00"#,
                output.file_name().unwrap().to_str().unwrap()
            );

            fs::write(&output_cue, test_cue_msf).unwrap();
        }

        let mut split_output =
            split(true, &output_cue, &Some(test_dir), true, &None, &cli).unwrap();

        let duration_rem = match test_msf {
            true => "",
            false => "\n    REM DURATION 44100",
        };

        let test_cue_multiple = format!(
            r#"REM COMPOSER "TEST"
TITLE "Album"
PERFORMER "Artist"
FILE "01 Artist - 1.wav" WAVE
  TRACK 01 AUDIO
    TITLE "1"
    PERFORMER "Artist"
    INDEX 01 00:00:00{duration_rem}
FILE "02 Artist - 2.wav" WAVE
  TRACK 02 AUDIO
    TITLE "2"
    PERFORMER "Artist"
    INDEX 01 00:00:00{duration_rem}
FILE "03 Artist - 3.wav" WAVE
  TRACK 03 AUDIO
    TITLE "3"
    PERFORMER "Artist"
    INDEX 01 00:00:00{duration_rem}"#
        );

        assert!(fs::read_to_string(output_cue_multiple).unwrap() == test_cue_multiple);

        if flac {
            let mut flac_output: Vec<PathBuf> = vec![];

            for wav_file in split_output {
                encode_to_flac(&wav_file);
                let mut flac_file = wav_file.parent().unwrap().to_path_buf();
                flac_file
                    .push(wav_file.file_stem().unwrap().to_str().unwrap().to_string() + ".flac");
                flac_output.push(flac_file);
            }

            split_output = flac_output;
        }

        check_file_hashes(&input, &split_output);
        remove_tmp_files(flac);
        remove_wavs(&split_output);
    }

    fn create_test_wavs(flac: bool) -> Vec<PathBuf> {
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
            test_file.push(format!("{i}.wav"));
            let mut writer = WavWriter::create(&test_file, spec).unwrap();

            for t in (0..88200).map(|x| x as f32 / 44100.0) {
                let sample = (t * 440.0 + i as f32 * 2.0 * std::f32::consts::PI).sin();
                let amplitude = i16::MAX as f32;
                writer.write_sample((sample * amplitude) as i16).unwrap();
            }

            writer.finalize().unwrap();

            if flac {
                encode_to_flac(&test_file);
                test_file.pop();
                test_file.push(format!("{i}.flac"));
            }

            input.push(test_file);
        }

        input
    }

    fn encode_to_flac(file: &PathBuf) {
        let output = Command::new("flac")
            .args(["--delete-input-file", file.to_str().unwrap()])
            .output()
            .expect("Failed to encode test wav to FLAC");

        assert!(output.status.success());
    }

    fn check_file_hashes(input: &Vec<PathBuf>, output: &Vec<PathBuf>) {
        assert_eq!(input.len(), output.len());

        let input_iter = input.iter();
        let mut output_iter = output.iter();

        input_iter.for_each(|a| {
            let b = output_iter.next().unwrap();
            assert_eq!(hash_file(a), hash_file(b))
        });
    }

    fn hash_file(file: &PathBuf) -> Hash {
        let mut hasher = blake3::Hasher::new();
        hasher.update_reader(File::open(file).unwrap()).unwrap();
        hasher.finalize()
    }

    fn get_test_dir() -> PathBuf {
        let mut test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_dir.push("tmp");

        if !fs::exists(&test_dir).unwrap() {
            fs::create_dir(&test_dir).unwrap();
        }

        test_dir
    }

    fn remove_tmp_files(flac: bool) {
        let test_dir = get_test_dir();

        let mut output = test_dir.clone();
        output.push("output.wav");

        let mut output_cue = test_dir.clone();
        output_cue.push("output.cue");

        let mut output_cue_multiple = test_dir.clone();
        output_cue_multiple.push("output_multiple.cue");

        for i in 1..=3 {
            let mut test_file = test_dir.clone();
            test_file.push(format!("{i}.wav"));

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

        if fs::exists(&output_cue_multiple).unwrap() {
            fs::remove_file(output_cue_multiple).unwrap();
        }

        if flac {
            for i in 1..=3 {
                let mut test_file = test_dir.clone();
                test_file.push(format!("{i}.flac"));

                if fs::exists(&test_file).unwrap() {
                    fs::remove_file(test_file).unwrap();
                }
            }

            let mut output_flac = test_dir.clone();
            output_flac.push("output.flac");

            if fs::exists(&output_flac).unwrap() {
                fs::remove_file(output_flac).unwrap();
            }
        }
    }

    fn remove_wavs(files: &Vec<PathBuf>) {
        for file in files {
            if (file.extension().unwrap() == "wav" || file.extension().unwrap() == "flac")
                && fs::exists(file).unwrap()
            {
                fs::remove_file(file).unwrap();
            }
        }
    }
}
