use super::envelope::EnvelopeFollower;
use super::gain::GainComputer;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompressorSettings {
    pub threshold_db: f32,
    pub ratio: f32,
    pub attack_ms: f32,
    pub release_ms: f32,
    pub makeup_db: f32,
}

impl Default for CompressorSettings {
    fn default() -> Self {
        Self {
            threshold_db: -18.0,
            ratio: 4.0,
            attack_ms: 10.0,
            release_ms: 100.0,
            makeup_db: 0.0,
        }
    }
}

pub struct PeakCompressor {
    pub threshold_db: f32,
    pub ratio: f32,
    pub attack_ms: f32,
    pub release_ms: f32,
    pub makeup_db: f32,
    follower: EnvelopeFollower,
}

impl PeakCompressor {
    pub fn new(sample_rate: f32) -> Self {
        let defaults = CompressorSettings::default();
        Self {
            threshold_db: defaults.threshold_db,
            ratio: defaults.ratio,
            attack_ms: defaults.attack_ms,
            release_ms: defaults.release_ms,
            makeup_db: defaults.makeup_db,
            follower: EnvelopeFollower::new(sample_rate, defaults.attack_ms, defaults.release_ms),
        }
    }

    pub fn with_settings(settings: CompressorSettings, sample_rate: f32) -> Self {
        let mut comp = Self::new(sample_rate);
        comp.apply_settings(&settings);
        comp
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.follower.set_sample_rate(sample_rate);
    }

    pub fn reset(&mut self) {
        self.follower.reset();
    }

    #[inline]
    pub fn envelope(&self) -> f32 {
        self.follower.current_envelope()
    }

    pub fn apply_settings(&mut self, settings: &CompressorSettings) {
        self.threshold_db = settings.threshold_db;
        self.ratio = settings.ratio;
        self.attack_ms = settings.attack_ms;
        self.release_ms = settings.release_ms;
        self.makeup_db = settings.makeup_db;
        self.follower.set_times(self.attack_ms, self.release_ms);
    }

    pub fn settings(&self) -> CompressorSettings {
        CompressorSettings {
            threshold_db: self.threshold_db,
            ratio: self.ratio,
            attack_ms: self.attack_ms,
            release_ms: self.release_ms,
            makeup_db: self.makeup_db,
        }
    }

    #[inline]
    pub fn next_gain(&mut self, peak: f32) -> f32 {
        self.follower.set_times(self.attack_ms, self.release_ms);
        let envelope = self.follower.process_peak(peak);

        let gain_computer = GainComputer::new(self.threshold_db, self.ratio, self.makeup_db);
        gain_computer.compute_linear_gain(envelope)
    }

    #[inline]
    pub fn process_sample(&mut self, sample: f32) -> f32 {
        let gain = self.next_gain(sample.abs());
        sample * gain
    }

    #[inline]
    pub fn process_frame(&mut self, frame: &mut [f32]) {
        let peak = frame
            .iter()
            .fold(0.0f32, |max, sample| max.max(sample.abs()));
        let gain = self.next_gain(peak);
        for sample in frame {
            *sample *= gain;
        }
    }

    pub fn process_interleaved(&mut self, buffer: &mut [f32], channels: usize) {
        if channels == 0 {
            return;
        }
        for frame in buffer.chunks_mut(channels) {
            self.process_frame(frame);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unity_gain_for_quiet_signal() {
        let mut compressor = PeakCompressor::new(44100.0);
        compressor.threshold_db = -10.0;
        compressor.ratio = 4.0;
        compressor.makeup_db = 0.0;

        let quiet_gain = compressor.next_gain(0.1);
        assert!((quiet_gain - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_gain_reduction_for_loud_signal() {
        let mut compressor = PeakCompressor::new(44100.0);
        compressor.threshold_db = -10.0;
        compressor.ratio = 4.0;
        compressor.attack_ms = 0.001;
        compressor.makeup_db = 0.0;

        for _ in 0..100 {
            compressor.next_gain(1.0);
        }
        let gain = compressor.next_gain(1.0);
        let expected_gain = 10f32.powf(-7.5 / 20.0);
        assert!((gain - expected_gain).abs() < 0.05);
    }

    #[test]
    fn test_process_interleaved() {
        let mut compressor = PeakCompressor::new(44100.0);
        let mut stereo_buf = vec![0.1, 0.1, 0.2, 0.2];
        compressor.process_interleaved(&mut stereo_buf, 2);
        assert!((stereo_buf[0] - 0.1).abs() < 1e-3);
        assert!((stereo_buf[1] - 0.1).abs() < 1e-3);
    }
}
