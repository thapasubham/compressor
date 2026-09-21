use nice_plug::prelude::*;
use std::sync::Arc;

pub mod dsp;

use dsp::PeakCompressor;

pub struct Compressor {
    params: Arc<CompressorParams>,
    dsp: PeakCompressor,
}

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
}

impl Default for Compressor {
    fn default() -> Self {
        Self {
            params: Arc::new(CompressorParams::default()),
            dsp: PeakCompressor::new(44_100.0),
        }
    }
}

impl Default for CompressorParams {
    fn default() -> Self {
        Self {
            threshold: FloatParam::new(
                "Threshold",
                -18.0,
                FloatRange::Linear {
                    min: -60.0,
                    max: 0.0,
                },
            )
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_rounded(1)),

            ratio: FloatParam::new(
                "Ratio",
                4.0,
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
                10.0,
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
                100.0,
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
                0.0,
                FloatRange::Linear {
                    min: -12.0,
                    max: 24.0,
                },
            )
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_rounded(1)),
        }
    }
}

impl Plugin for Compressor {
    const NAME: &'static str = "Compressor";
    const VENDOR: &'static str = "Your Name";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "info@example.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type Editor = ();
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn activate(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl ActivateContext<Self>,
    ) -> bool {
        self.dsp.set_sample_rate(buffer_config.sample_rate);
        self.dsp.reset();
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.dsp.threshold_db = self.params.threshold.value();
        self.dsp.ratio = self.params.ratio.value();
        self.dsp.attack_ms = self.params.attack.value();
        self.dsp.release_ms = self.params.release.value();
        self.dsp.makeup_db = self.params.makeup.value();

        for mut channel_samples in buffer.iter_samples() {
            // Use the loudest channel to drive a single, stereo-linked envelope.
            let peak = channel_samples
                .iter_mut()
                .fold(0.0f32, |max, sample| max.max(sample.abs()));

            let gain = self.dsp.next_gain(peak);

            for sample in channel_samples {
                *sample *= gain;
            }
        }

        ProcessStatus::Normal
    }

    fn deactivate(&mut self) {}
}

impl ClapPlugin for Compressor {
    const CLAP_ID: &'static str = "com.example.compressor";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A basic peak compressor");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Compressor,
    ];
}

impl Vst3Plugin for Compressor {
    const VST3_CLASS_ID: [u8; 16] = *b"CompressorPlugin";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nice_export_clap!(Compressor);
nice_export_vst3!(Compressor);
