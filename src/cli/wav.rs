use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use std::path::Path;

pub struct WavAudio {
    pub spec: WavSpec,
    pub samples: Vec<f32>,
}

impl WavAudio {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path_ref = path.as_ref();
        let mut reader = WavReader::open(path_ref)
            .map_err(|e| format!("Failed to open '{}': {e}", path_ref.display()))?;

        let spec = reader.spec();
        let samples: Vec<f32> = match (spec.sample_format, spec.bits_per_sample) {
            (SampleFormat::Float, 32) => {
                let mut out = Vec::new();
                for sample in reader.samples::<f32>() {
                    let s = sample.map_err(|e| format!("Failed reading float sample: {e}"))?;
                    out.push(s);
                }
                out
            }
            (SampleFormat::Int, bits) if bits > 0 && bits <= 32 => {
                let max = (1i64 << (bits - 1)) as f32;
                let mut out = Vec::new();
                for sample in reader.samples::<i32>() {
                    let s = sample.map_err(|e| format!("Failed reading int sample: {e}"))?;
                    out.push(s as f32 / max);
                }
                out
            }
            (format, bits) => {
                return Err(format!(
                    "Unsupported WAV format: {format:?} @ {bits} bits"
                ));
            }
        };

        Ok(Self { spec, samples })
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let path_ref = path.as_ref();
        let mut writer = WavWriter::create(path_ref, self.spec)
            .map_err(|e| format!("Failed to create '{}': {e}", path_ref.display()))?;

        match (self.spec.sample_format, self.spec.bits_per_sample) {
            (SampleFormat::Float, 32) => {
                for &s in &self.samples {
                    writer
                        .write_sample(s)
                        .map_err(|e| format!("Failed to write sample: {e}"))?;
                }
            }
            (SampleFormat::Int, bits) if bits > 0 && bits <= 32 => {
                let max = (1i64 << (bits - 1)) as f32 - 1.0;
                for &s in &self.samples {
                    let clamped = (s * max).clamp(-max - 1.0, max) as i32;
                    writer
                        .write_sample(clamped)
                        .map_err(|e| format!("Failed to write sample: {e}"))?;
                }
            }
            (format, bits) => {
                return Err(format!(
                    "Unsupported target WAV format: {format:?} @ {bits} bits"
                ));
            }
        }

        writer
            .finalize()
            .map_err(|e| format!("Failed to finalize '{}': {e}", path_ref.display()))?;

        Ok(())
    }
}
