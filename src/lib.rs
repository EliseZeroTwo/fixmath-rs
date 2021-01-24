#[allow(dead_code)]

pub fn no_rounding() -> bool {
    #[cfg(feature = "no-rounding")]
    return true;
    false
}

mod consts;
pub mod fix16;

#[cfg(test)]
mod tests {
    use crate::fix16::{FIX16_MAX, FIX16_MIN, FIX16_ONE, Fix16};

    extern crate libfixmath_src_rs;
    use libfixmath_src_rs::*;

    #[test]
    fn basic_ops() {
        let rusty_twenty = Fix16::from(20);
        let native_twenty = fix16_from_int(20);
        assert_eq!(rusty_twenty.0, native_twenty);

        let rusty_two_thousand_thirty_nine_dot_four_two = Fix16::from(2039.42);
        let native_two_thousand_thirty_nine_dot_four_two = fix16_from_float(2039.42);
        assert_eq!(rusty_two_thousand_thirty_nine_dot_four_two.0, native_two_thousand_thirty_nine_dot_four_two);

        let rusty_add = rusty_two_thousand_thirty_nine_dot_four_two + rusty_twenty;
        let native_add;
        unsafe {
            native_add = fix16_add(native_two_thousand_thirty_nine_dot_four_two, native_twenty);
        }
        assert_eq!(rusty_add.0, native_add);

        let rusty_sub = rusty_two_thousand_thirty_nine_dot_four_two - rusty_twenty;
        let native_sub;
        unsafe {
            native_sub = fix16_sub(native_two_thousand_thirty_nine_dot_four_two, native_twenty);
        }
        assert_eq!(rusty_sub.0, native_sub);

        let rusty_mul = rusty_two_thousand_thirty_nine_dot_four_two * rusty_twenty;
        let native_mul;
        unsafe {
            native_mul = fix16_mul(native_two_thousand_thirty_nine_dot_four_two, native_twenty);
        }
        assert_eq!(rusty_mul.0, native_mul);

        let rusty_div = rusty_two_thousand_thirty_nine_dot_four_two / rusty_twenty;
        let native_div;
        unsafe {
            native_div = fix16_div(native_two_thousand_thirty_nine_dot_four_two, native_twenty);
        }
        assert_eq!(rusty_div.0, native_div);

        let rusty_sin = rusty_two_thousand_thirty_nine_dot_four_two.sin();
        let native_sin;
        unsafe {
            native_sin = fix16_sin(native_two_thousand_thirty_nine_dot_four_two);
        }
        assert_eq!(rusty_sin.0, native_sin);

        let rusty_cos = rusty_two_thousand_thirty_nine_dot_four_two.cos();
        let native_cos;
        unsafe {
            native_cos = fix16_cos(native_two_thousand_thirty_nine_dot_four_two);
        }
        assert_eq!(rusty_cos.0, native_cos);

        let rusty_tan = rusty_two_thousand_thirty_nine_dot_four_two.tan();
        let native_tan;
        unsafe {
            native_tan = fix16_tan(native_two_thousand_thirty_nine_dot_four_two);
        }
        assert_eq!(rusty_tan.0, native_tan);
    }

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

        let hex_str_parse = Fix16::from_hex_str(&"0x32.69".to_string());
        if let Err(why) = hex_str_parse {
            panic!(why);
        }
        let hex_str_parse = hex_str_parse.unwrap();
        assert_eq!(hex_str_parse, Fix16(0x0032_6900));

        let str_parse = Fix16::from_str(&"-69.420".to_string());
        if let Err(why) = str_parse {
            panic!(why);
        }
        let str_parse = str_parse.unwrap();
        assert_eq!(str_parse, Fix16::from(-69.420));

        assert_eq!(Fix16::from(22).sqrt(), Fix16(0x4b0bf));
        assert_eq!(Fix16::from(2451.1238).sqrt(), Fix16(0x318242));

        assert_eq!(Fix16::from(203).sin(), Fix16(0xeee4));
        assert_eq!(Fix16::from(4203).cos(), Fix16(0xe758));
        assert_eq!(Fix16::from(2193).tan(), Fix16(0x2cac));
    }
}
