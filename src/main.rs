//! Standalone runner for quickly testing the plugin's GUI and audio processing
//! outside of a DAW. Connects directly to the system's audio ports.

use compressor::Compressor;
use nice_plug::prelude::nice_export_standalone;

fn main() {
    nice_export_standalone::<Compressor>();
}
