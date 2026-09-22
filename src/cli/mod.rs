pub mod args;
pub mod wav;

pub use args::{CliArgs, CliError};
pub use wav::WavAudio;

use crate::dsp::PeakCompressor;
use std::env;

pub fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    run_with_args(&args)
}

pub fn run_with_args(args: &[String]) -> Result<(), String> {
    let cli_args = match CliArgs::parse(args) {
        Ok(parsed) => parsed,
        Err(CliError::HelpRequested) => {
            println!("{}", args::USAGE);
            return Ok(());
        }
        Err(err) => {
            return Err(err.to_string());
        }
    };

    println!("Loading audio from '{}'...", cli_args.input_path);
    let mut audio = WavAudio::load(&cli_args.input_path)?;

    let sample_rate = audio.spec.sample_rate as f32;
    let channels = audio.spec.channels as usize;

    println!(
        "Audio spec: {} Hz, {} ch, {:?} @ {} bits",
        audio.spec.sample_rate,
        audio.spec.channels,
        audio.spec.sample_format,
        audio.spec.bits_per_sample
    );

    let mut compressor = PeakCompressor::with_settings(cli_args.settings, sample_rate);

    println!(
        "Processing with settings: thresh={:.1}dB, ratio={:.1}:1, attack={:.1}ms, release={:.1}ms, makeup={:.1}dB",
        cli_args.settings.threshold_db,
        cli_args.settings.ratio,
        cli_args.settings.attack_ms,
        cli_args.settings.release_ms,
        cli_args.settings.makeup_db
    );

    compressor.process_interleaved(&mut audio.samples, channels);

    println!("Saving compressed audio to '{}'...", cli_args.output_path);
    audio.save(&cli_args.output_path)?;

    println!("Done! Wrote '{}'", cli_args.output_path);
    Ok(())
}
