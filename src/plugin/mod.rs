pub mod params;

pub use params::CompressorParams;

use crate::dsp::{gain_to_db, PeakCompressor};
use crate::ui;
use nice_plug::prelude::*;
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub struct Compressor {
    params: Arc<CompressorParams>,
    dsp: PeakCompressor,
}

impl Default for Compressor {
    fn default() -> Self {
        Self {
            params: Arc::new(CompressorParams::default()),
            dsp: PeakCompressor::new(44_100.0),
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

    type Editor = nice_plug_iced::IcedEditor;
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Self::Editor> {
        ui::create(self.params.clone())
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
        let settings = self.params.to_settings();
        self.dsp.apply_settings(&settings);

        let bypassed = self.params.bypass.value();
        let mix = self.params.mix.value();

        for mut channel_samples in buffer.iter_samples() {
            let peak = channel_samples
                .iter_mut()
                .fold(0.0f32, |max, sample| max.max(sample.abs()));

            let gain = self.dsp.next_gain(peak);
            let gain_reduction_db = if bypassed {
                0.0
            } else {
                (gain_to_db(gain) - settings.makeup_db).min(0.0)
            };
            self.params
                .gain_reduction
                .store(gain_reduction_db, Ordering::Relaxed);

            if bypassed {
                continue;
            }

            for sample in channel_samples {
                let dry = *sample;
                let wet = dry * gain;
                *sample = dry + (wet - dry) * mix;
            }
        }

        ProcessStatus::Normal
    }

    fn deactivate(&mut self) {}
}

impl ClapPlugin for Compressor {
    const CLAP_ID: &'static str = "Cha Cha Compressor";
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
