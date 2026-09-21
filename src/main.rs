use compressor::dsp::PeakCompressor;
use hound::{SampleFormat, WavReader, WavWriter};
use std::env;
use std::process;

fn print_usage() {
    eprintln!("Usage: compressor <input.wav> <output.wav> [options]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --threshold <dB>   Threshold in dB (default: -18.0)");
    eprintln!("  --ratio <n>        Compression ratio, e.g. 4 for 4:1 (default: 4.0)");
    eprintln!("  --attack <ms>      Attack time in milliseconds (default: 10.0)");
    eprintln!("  --release <ms>     Release time in milliseconds (default: 100.0)");
    eprintln!("  --makeup <dB>      Makeup gain in dB (default: 0.0)");
}

fn parse_flag(args: &[String], name: &str, default: f32) -> f32 {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(default)
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() < 2 || args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        process::exit(if args.is_empty() { 1 } else { 0 });
    }

    let input_path = &args[0];
    let output_path = &args[1];

    let mut reader = WavReader::open(input_path).unwrap_or_else(|e| {
        eprintln!("Failed to open '{input_path}': {e}");
        process::exit(1);
    });
    let spec = reader.spec();

    let mut compressor = PeakCompressor::new(spec.sample_rate as f32);
    compressor.threshold_db = parse_flag(&args, "--threshold", -18.0);
    compressor.ratio = parse_flag(&args, "--ratio", 4.0);
    compressor.attack_ms = parse_flag(&args, "--attack", 10.0);
    compressor.release_ms = parse_flag(&args, "--release", 100.0);
    compressor.makeup_db = parse_flag(&args, "--makeup", 0.0);

    let samples: Vec<f32> = match (spec.sample_format, spec.bits_per_sample) {
        (SampleFormat::Float, 32) => reader
            .samples::<f32>()
            .map(|s| s.expect("failed to read sample"))
            .collect(),
        (SampleFormat::Int, bits) => {
            let max = (1i64 << (bits - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|s| s.expect("failed to read sample") as f32 / max)
                .collect()
        }
        (format, bits) => {
            eprintln!("Unsupported WAV format: {format:?} @ {bits} bits");
            process::exit(1);
        }
    };

    let channels = spec.channels as usize;
    let mut output_samples = vec![0.0f32; samples.len()];

    for (in_frame, out_frame) in samples
        .chunks(channels)
        .zip(output_samples.chunks_mut(channels))
    {
        let peak = in_frame.iter().fold(0.0f32, |m, s| m.max(s.abs()));
        let gain = compressor.next_gain(peak);
        for (o, i) in out_frame.iter_mut().zip(in_frame.iter()) {
            *o = i * gain;
        }
    }

    let mut writer = WavWriter::create(output_path, spec).unwrap_or_else(|e| {
        eprintln!("Failed to create '{output_path}': {e}");
        process::exit(1);
    });

    match (spec.sample_format, spec.bits_per_sample) {
        (SampleFormat::Float, 32) => {
            for s in &output_samples {
                writer.write_sample(*s).expect("failed to write sample");
            }
        }
        (SampleFormat::Int, bits) => {
            let max = (1i64 << (bits - 1)) as f32 - 1.0;
            for s in &output_samples {
                let clamped = (s * max).clamp(-max - 1.0, max) as i32;
                writer
                    .write_sample(clamped)
                    .expect("failed to write sample");
            }
        }
        _ => unreachable!("validated when reading input samples"),
    }

    writer.finalize().unwrap_or_else(|e| {
        eprintln!("Failed to finalize '{output_path}': {e}");
        process::exit(1);
    });

    println!("Wrote '{output_path}'");
}
