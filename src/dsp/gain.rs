use super::units::{db_to_gain, gain_to_db};

#[derive(Debug, Clone, PartialEq)]
pub struct GainComputer {
    pub threshold_db: f32,
    pub ratio: f32,
    pub makeup_db: f32,
}

impl GainComputer {
    pub fn new(threshold_db: f32, ratio: f32, makeup_db: f32) -> Self {
        Self {
            threshold_db,
            ratio: ratio.max(1.0),
            makeup_db,
        }
    }

    #[inline]
    pub fn compute_gain_reduction_db(&self, envelope_db: f32) -> f32 {
        if envelope_db > self.threshold_db && self.ratio > 1.0 {
            (envelope_db - self.threshold_db) * (1.0 / self.ratio - 1.0)
        } else {
            0.0
        }
    }

    #[inline]
    pub fn compute_linear_gain(&self, envelope: f32) -> f32 {
        let envelope_db = gain_to_db(envelope);
        let gr_db = self.compute_gain_reduction_db(envelope_db);
        db_to_gain(gr_db + self.makeup_db)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_below_threshold_unity_reduction() {
        let computer = GainComputer::new(-10.0, 4.0, 0.0);
        let gr_db = computer.compute_gain_reduction_db(-15.0);
        assert_eq!(gr_db, 0.0);

        let gain = computer.compute_linear_gain(db_to_gain(-15.0));
        assert!((gain - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_above_threshold_compression() {
        let computer = GainComputer::new(-10.0, 4.0, 0.0);
        let gr_db = computer.compute_gain_reduction_db(-2.0);
        assert!((gr_db - (-6.0)).abs() < 1e-5);

        let gain = computer.compute_linear_gain(db_to_gain(-2.0));
        let expected_gain = db_to_gain(-6.0);
        assert!((gain - expected_gain).abs() < 1e-5);
    }

    #[test]
    fn test_makeup_gain() {
        let computer = GainComputer::new(-10.0, 4.0, 6.0);
        let gain = computer.compute_linear_gain(db_to_gain(-20.0));
        let expected = db_to_gain(6.0);
        assert!((gain - expected).abs() < 1e-5);
    }

    #[test]
    fn test_ratio_1_to_1() {
        let computer = GainComputer::new(-10.0, 1.0, 0.0);
        let gr_db = computer.compute_gain_reduction_db(0.0);
        assert_eq!(gr_db, 0.0);
    }
}
