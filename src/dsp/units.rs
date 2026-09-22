pub const MIN_GAIN_FLOOR: f32 = 1e-6;

#[inline]
pub fn db_to_gain(db: f32) -> f32 {
    10.0f32.powf(db / 20.0)
}

#[inline]
pub fn gain_to_db(gain: f32) -> f32 {
    20.0 * gain.max(MIN_GAIN_FLOOR).log10()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_to_gain_unity() {
        assert!((db_to_gain(0.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_db_to_gain_minus_six_db() {
        let gain = db_to_gain(-6.0206);
        assert!((gain - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_gain_to_db_roundtrip() {
        for &db in &[-60.0, -24.0, -12.0, -6.0, 0.0, 6.0, 12.0] {
            let gain = db_to_gain(db);
            let roundtrip_db = gain_to_db(gain);
            assert!((db - roundtrip_db).abs() < 1e-4);
        }
    }

    #[test]
    fn test_gain_to_db_zero_or_negative() {
        let db_zero = gain_to_db(0.0);
        assert!(db_zero.is_finite());
        assert!((db_zero - gain_to_db(MIN_GAIN_FLOOR)).abs() < 1e-6);
    }
}
