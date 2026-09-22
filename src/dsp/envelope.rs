#[derive(Debug, Clone, PartialEq)]
pub struct EnvelopeFollower {
    sample_rate: f32,
    attack_ms: f32,
    release_ms: f32,
    attack_coeff: f32,
    release_coeff: f32,
    envelope: f32,
}

impl EnvelopeFollower {
    pub fn new(sample_rate: f32, attack_ms: f32, release_ms: f32) -> Self {
        let mut follower = Self {
            sample_rate: sample_rate.max(1.0),
            attack_ms: attack_ms.max(0.001),
            release_ms: release_ms.max(0.001),
            attack_coeff: 0.0,
            release_coeff: 0.0,
            envelope: 0.0,
        };
        follower.update_coeffs();
        follower
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
        self.update_coeffs();
    }

    pub fn set_attack_ms(&mut self, attack_ms: f32) {
        self.attack_ms = attack_ms.max(0.001);
        self.attack_coeff = Self::calc_coeff(self.attack_ms, self.sample_rate);
    }

    pub fn set_release_ms(&mut self, release_ms: f32) {
        self.release_ms = release_ms.max(0.001);
        self.release_coeff = Self::calc_coeff(self.release_ms, self.sample_rate);
    }

    pub fn set_times(&mut self, attack_ms: f32, release_ms: f32) {
        self.attack_ms = attack_ms.max(0.001);
        self.release_ms = release_ms.max(0.001);
        self.update_coeffs();
    }

    pub fn reset(&mut self) {
        self.envelope = 0.0;
    }

    #[inline]
    pub fn current_envelope(&self) -> f32 {
        self.envelope
    }

    #[inline]
    pub fn process_peak(&mut self, peak: f32) -> f32 {
        let coeff = if peak > self.envelope {
            self.attack_coeff
        } else {
            self.release_coeff
        };

        self.envelope = coeff * self.envelope + (1.0 - coeff) * peak;
        self.envelope
    }

    #[inline]
    fn calc_coeff(time_ms: f32, sample_rate: f32) -> f32 {
        (-1.0 / (time_ms * 0.001 * sample_rate)).exp()
    }

    fn update_coeffs(&mut self) {
        self.attack_coeff = Self::calc_coeff(self.attack_ms, self.sample_rate);
        self.release_coeff = Self::calc_coeff(self.release_ms, self.sample_rate);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_attack_rise() {
        let mut follower = EnvelopeFollower::new(44100.0, 10.0, 100.0);
        assert_eq!(follower.current_envelope(), 0.0);

        let mut prev = 0.0;
        for _ in 0..100 {
            let env = follower.process_peak(1.0);
            assert!(env > prev);
            assert!(env <= 1.0);
            prev = env;
        }
    }

    #[test]
    fn test_envelope_release_decay() {
        let mut follower = EnvelopeFollower::new(44100.0, 1.0, 100.0);

        for _ in 0..2000 {
            follower.process_peak(1.0);
        }
        let charged = follower.current_envelope();
        assert!((charged - 1.0).abs() < 1e-2);

        let mut prev = charged;
        for _ in 0..100 {
            let env = follower.process_peak(0.0);
            assert!(env < prev);
            assert!(env >= 0.0);
            prev = env;
        }
    }

    #[test]
    fn test_reset() {
        let mut follower = EnvelopeFollower::new(44100.0, 10.0, 100.0);
        follower.process_peak(1.0);
        assert!(follower.current_envelope() > 0.0);

        follower.reset();
        assert_eq!(follower.current_envelope(), 0.0);
    }
}
