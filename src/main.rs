use std::process;

fn main() {
    if let Err(err) = compressor::cli::run() {
        eprintln!("Error: {err}");
        process::exit(1);
    }
}
