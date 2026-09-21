pub struct PeakCompressor {
    pub threshold_db: f32,
    pub ratio: f32,
    pub attack_ms: f32,
    pub release_ms: f32,
    pub makeup_db: f32,
    sample_rate: f32,
    envelope: f32,
}

impl PeakCompressor {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            threshold_db: -18.0,
            ratio: 4.0,
            attack_ms: 10.0,
            release_ms: 100.0,
            makeup_db: 0.0,
            sample_rate,
            envelope: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    pub fn reset(&mut self) {
        self.envelope = 0.0;
    }

    pub fn next_gain(&mut self, peak: f32) -> f32 {
        let attack_coeff = (-1.0 / (self.attack_ms * 0.001 * self.sample_rate)).exp();
        let release_coeff = (-1.0 / (self.release_ms * 0.001 * self.sample_rate)).exp();
        let coeff = if peak > self.envelope {
            attack_coeff
        } else {
            release_coeff
        };
        self.envelope = coeff * self.envelope + (1.0 - coeff) * peak;

        let envelope_db = gain_to_db(self.envelope.max(1e-6));
        let gain_reduction_db = if envelope_db > self.threshold_db {
            (envelope_db - self.threshold_db) * (1.0 / self.ratio - 1.0)
        } else {
            0.0
        };
        db_to_gain(gain_reduction_db) * db_to_gain(self.makeup_db)
    }
}

pub fn db_to_gain(db: f32) -> f32 {
    10f32.powf(db / 20.0)
}

pub fn gain_to_db(gain: f32) -> f32 {
    20.0 * gain.log10()
}
