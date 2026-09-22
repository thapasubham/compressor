use crate::dsp::CompressorSettings;
use nice_plug::prelude::*;
use std::sync::Arc;

#[derive(Params)]
pub struct CompressorParams {
    #[id = "threshold"]
    pub threshold: FloatParam,

    #[id = "ratio"]
    pub ratio: FloatParam,

    #[id = "attack"]
    pub attack: FloatParam,

    #[id = "release"]
    pub release: FloatParam,

    #[id = "makeup"]
    pub makeup: FloatParam,

    #[id = "mix"]
    pub mix: FloatParam,

    #[id = "bypass"]
    pub bypass: BoolParam,

    #[persist = "gain_reduction"]
    pub gain_reduction: Arc<AtomicF32>,

    #[persist = "input_level"]
    pub input_level: Arc<AtomicF32>,

    #[persist = "output_level"]
    pub output_level: Arc<AtomicF32>,
}

impl Default for CompressorParams {
    fn default() -> Self {
        let defaults = CompressorSettings::default();

        Self {
            threshold: FloatParam::new(
                "Threshold",
                defaults.threshold_db,
                FloatRange::Linear {
                    min: -60.0,
                    max: 0.0,
                },
            )
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_rounded(1)),

            ratio: FloatParam::new(
                "Ratio",
                defaults.ratio,
                FloatRange::Skewed {
                    min: 1.0,
                    max: 20.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_unit(":1")
            .with_value_to_string(formatters::v2s_f32_rounded(1)),

            attack: FloatParam::new(
                "Attack",
                defaults.attack_ms,
                FloatRange::Skewed {
                    min: 0.1,
                    max: 500.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" ms")
            .with_value_to_string(formatters::v2s_f32_rounded(1)),

            release: FloatParam::new(
                "Release",
                defaults.release_ms,
                FloatRange::Skewed {
                    min: 1.0,
                    max: 1000.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" ms")
            .with_value_to_string(formatters::v2s_f32_rounded(1)),

            makeup: FloatParam::new(
                "Makeup Gain",
                defaults.makeup_db,
                FloatRange::Linear {
                    min: -12.0,
                    max: 24.0,
                },
            )
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_rounded(1)),

            mix: FloatParam::new(
                "Mix",
                1.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 1.0,
                },
            )
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            bypass: BoolParam::new("Bypass", false).make_bypass(),

            gain_reduction: Arc::new(AtomicF32::new(0.0)),
            input_level: Arc::new(AtomicF32::new(f32::NEG_INFINITY)),
            output_level: Arc::new(AtomicF32::new(f32::NEG_INFINITY)),
        }
    }
}

impl CompressorParams {
    pub fn to_settings(&self) -> CompressorSettings {
        CompressorSettings {
            threshold_db: self.threshold.value(),
            ratio: self.ratio.value(),
            attack_ms: self.attack.value(),
            release_ms: self.release.value(),
            makeup_db: self.makeup.value(),
        }
    }
}
