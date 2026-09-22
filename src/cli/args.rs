use crate::dsp::CompressorSettings;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum CliError {
    HelpRequested,
    MissingArguments,
    InvalidValue { flag: String, message: String },
    UnknownOption(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::HelpRequested => write!(f, "{}", USAGE),
            CliError::MissingArguments => {
                write!(f, "Missing required arguments.\n\n{}", USAGE)
            }
            CliError::InvalidValue { flag, message } => {
                write!(f, "Invalid value for {flag}: {message}")
            }
            CliError::UnknownOption(opt) => {
                write!(f, "Unknown option '{opt}'.\n\n{}", USAGE)
            }
        }
    }
}

pub const USAGE: &str = "\
Usage: compressor <input.wav> <output.wav> [options]

Options:
  --threshold <dB>   Threshold in dB (default: -18.0)
  --ratio <n>        Compression ratio, e.g. 4 for 4:1 (default: 4.0, min: 1.0)
  --attack <ms>      Attack time in milliseconds (default: 10.0, min: 0.1)
  --release <ms>     Release time in milliseconds (default: 100.0, min: 1.0)
  --makeup <dB>      Makeup gain in dB (default: 0.0)
  -h, --help         Print this help message";

#[derive(Debug, PartialEq)]
pub struct CliArgs {
    pub input_path: String,
    pub output_path: String,
    pub settings: CompressorSettings,
}

impl CliArgs {
    pub fn parse(args: &[String]) -> Result<Self, CliError> {
        if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
            return Err(CliError::HelpRequested);
        }

        if args.len() < 2 {
            return Err(CliError::MissingArguments);
        }

        let input_path = args[0].clone();
        let output_path = args[1].clone();

        if input_path.starts_with('-') || output_path.starts_with('-') {
            return Err(CliError::MissingArguments);
        }

        let mut settings = CompressorSettings::default();
        let mut idx = 2;

        while idx < args.len() {
            let flag = &args[idx];
            match flag.as_str() {
                "--threshold" => {
                    idx += 1;
                    let val = parse_f32(args, idx, flag)?;
                    settings.threshold_db = val;
                }
                "--ratio" => {
                    idx += 1;
                    let val = parse_f32(args, idx, flag)?;
                    if val < 1.0 {
                        return Err(CliError::InvalidValue {
                            flag: flag.clone(),
                            message: "Ratio must be at least 1.0".to_string(),
                        });
                    }
                    settings.ratio = val;
                }
                "--attack" => {
                    idx += 1;
                    let val = parse_f32(args, idx, flag)?;
                    if val <= 0.0 {
                        return Err(CliError::InvalidValue {
                            flag: flag.clone(),
                            message: "Attack must be greater than 0 ms".to_string(),
                        });
                    }
                    settings.attack_ms = val;
                }
                "--release" => {
                    idx += 1;
                    let val = parse_f32(args, idx, flag)?;
                    if val <= 0.0 {
                        return Err(CliError::InvalidValue {
                            flag: flag.clone(),
                            message: "Release must be greater than 0 ms".to_string(),
                        });
                    }
                    settings.release_ms = val;
                }
                "--makeup" => {
                    idx += 1;
                    let val = parse_f32(args, idx, flag)?;
                    settings.makeup_db = val;
                }
                other => {
                    return Err(CliError::UnknownOption(other.to_string()));
                }
            }
            idx += 1;
        }

        Ok(Self {
            input_path,
            output_path,
            settings,
        })
    }
}

fn parse_f32(args: &[String], index: usize, flag: &str) -> Result<f32, CliError> {
    let raw = args.get(index).ok_or_else(|| CliError::InvalidValue {
        flag: flag.to_string(),
        message: "Missing numeric value".to_string(),
    })?;

    raw.parse::<f32>().map_err(|_| CliError::InvalidValue {
        flag: flag.to_string(),
        message: format!("'{raw}' is not a valid number"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_help() {
        let args = vec!["--help".to_string()];
        assert_eq!(CliArgs::parse(&args), Err(CliError::HelpRequested));

        let args_short = vec!["-h".to_string()];
        assert_eq!(CliArgs::parse(&args_short), Err(CliError::HelpRequested));
    }

    #[test]
    fn test_parse_defaults() {
        let args = vec!["in.wav".to_string(), "out.wav".to_string()];
        let parsed = CliArgs::parse(&args).expect("should succeed");
        assert_eq!(parsed.input_path, "in.wav");
        assert_eq!(parsed.output_path, "out.wav");
        assert_eq!(parsed.settings, CompressorSettings::default());
    }

    #[test]
    fn test_parse_custom_flags() {
        let args = vec![
            "in.wav".to_string(),
            "out.wav".to_string(),
            "--threshold".to_string(),
            "-24.0".to_string(),
            "--ratio".to_string(),
            "8.0".to_string(),
            "--attack".to_string(),
            "5.0".to_string(),
            "--release".to_string(),
            "50.0".to_string(),
            "--makeup".to_string(),
            "3.0".to_string(),
        ];
        let parsed = CliArgs::parse(&args).expect("should succeed");
        assert_eq!(parsed.settings.threshold_db, -24.0);
        assert_eq!(parsed.settings.ratio, 8.0);
        assert_eq!(parsed.settings.attack_ms, 5.0);
        assert_eq!(parsed.settings.release_ms, 50.0);
        assert_eq!(parsed.settings.makeup_db, 3.0);
    }

    #[test]
    fn test_parse_invalid_ratio() {
        let args = vec![
            "in.wav".to_string(),
            "out.wav".to_string(),
            "--ratio".to_string(),
            "0.5".to_string(),
        ];
        assert!(matches!(
            CliArgs::parse(&args),
            Err(CliError::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_parse_missing_flag_value() {
        let args = vec![
            "in.wav".to_string(),
            "out.wav".to_string(),
            "--threshold".to_string(),
        ];
        assert!(matches!(
            CliArgs::parse(&args),
            Err(CliError::InvalidValue { .. })
        ));
    }
}
