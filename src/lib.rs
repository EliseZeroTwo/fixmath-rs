#[allow(dead_code)]

pub fn no_rounding() -> bool {
    #[cfg(feature = "no-rounding")]
    return true;
    false
}

pub mod fix16;

#[cfg(test)]
mod tests {
    use crate::fix16::{Fix16, FIX16_MAX, FIX16_MIN, FIX16_ONE};

    #[test]
    fn it_works() {
        let pos_two = Fix16::from(2);
        let neg_onetwothreefour_fivesix = Fix16::from(-1234.56);

        assert_eq!(f32::from(neg_onetwothreefour_fivesix + pos_two), -1232.56);
        assert_eq!(f32::from(neg_onetwothreefour_fivesix - pos_two), -1236.56);
        assert_eq!(f32::from(neg_onetwothreefour_fivesix * pos_two), -2469.12);
        assert_eq!(f32::from(neg_onetwothreefour_fivesix / pos_two), -617.28);

        assert_eq!(
            f32::from(neg_onetwothreefour_fivesix.overflowing_add(pos_two).0),
            -1232.56
        );
        assert_eq!(
            f32::from(neg_onetwothreefour_fivesix.overflowing_sub(pos_two).0),
            -1236.56
        );
        assert_eq!(
            f32::from(neg_onetwothreefour_fivesix.overflowing_mul(pos_two).0),
            -2469.12
        );
        assert_eq!(
            f32::from(neg_onetwothreefour_fivesix.overflowing_div(pos_two).0),
            -617.28
        );

        assert_eq!(
            f32::from(neg_onetwothreefour_fivesix.saturating_add(pos_two)),
            -1232.56
        );
        assert_eq!(
            f32::from(neg_onetwothreefour_fivesix.saturating_sub(pos_two)),
            -1236.56
        );
        assert_eq!(
            f32::from(neg_onetwothreefour_fivesix.saturating_mul(pos_two)),
            -2469.12
        );
        assert_eq!(
            f32::from(neg_onetwothreefour_fivesix.saturating_div(pos_two)),
            -617.28
        );

        let overflowing_operation = FIX16_MAX.overflowing_add(FIX16_ONE);
        assert_eq!(overflowing_operation.1, true);
        assert_eq!(overflowing_operation.0 - Fix16(FIX16_ONE.0 - 1), FIX16_MIN);

        let underflowing_operation = FIX16_MIN.overflowing_sub(FIX16_ONE - Fix16(FIX16_ONE.0 - 1));
        assert_eq!(underflowing_operation.1, true);
        assert_eq!(underflowing_operation.0, FIX16_MAX);
    }
}
