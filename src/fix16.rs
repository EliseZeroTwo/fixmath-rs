use std::{ops, u16, u32};

#[derive(Copy, Clone, Debug)]
pub struct Fix16(pub i32);

pub const FOUR_DIV_PI: Fix16 = Fix16(0x145F3);
pub const NEG_FOUR_DIV_PI: Fix16 = Fix16(-0x67c0);
pub const X4_CORRECTION_COMPONENT: Fix16 = Fix16(0x399A);
pub const PI_DIV_4: Fix16 = Fix16(0x0000C90F);
pub const THREE_PI_DIV_4: Fix16 = Fix16(0x00025B2F);

pub const FIX16_MAX: Fix16 = Fix16(0x7FFFFFFF);
pub const FIX16_MIN: Fix16 = Fix16(-0x80000000);
pub const FIX16_OVERFLOW: Fix16 = FIX16_MIN;

pub const FIX16_PI: Fix16 = Fix16(205887);
pub const FIX16_E: Fix16 = Fix16(178145);
pub const FIX16_ONE: Fix16 = Fix16(0x00010000);

impl From<i32> for Fix16 {
    fn from(val: i32) -> Self {
        Fix16(val * FIX16_ONE.0)
    }
}

impl From<f32> for Fix16 {
    fn from(val: f32) -> Self {
        Fix16({
            let mut x = val * FIX16_ONE.0 as f32;

            if !crate::no_rounding() {
                x += if x.is_sign_positive() { 0.5 } else { -0.5 };
            }

            x as i32
        })
    }
}

impl From<Fix16> for i32 {
    fn from(val: Fix16) -> Self {
        if crate::no_rounding() {
            val.0 >> 16
        } else if val.0 >= 0 {
            (val.0 + (FIX16_ONE.0 >> 1)) / FIX16_ONE.0
        } else {
            (val.0 - (FIX16_ONE.0 >> 1)) / FIX16_ONE.0
        }
    }
}

impl From<Fix16> for f32 {
    fn from(val: Fix16) -> Self {
        val.0 as f32 / FIX16_ONE.0 as f32
    }
}

impl Fix16 {
    pub fn abs(self) -> Fix16 {
        if self.0.is_negative() {
            Fix16(-self.0)
        } else {
            self
        }
    }

    pub fn floor(self) -> Fix16 {
        Fix16(self.0 & -0x10000)
    }

    pub fn ceil(self) -> Fix16 {
        Fix16(
            (self.0 & -0x10000)
                + if (self.0 & 0xFFFF) != 0 {
                    FIX16_ONE.0
                } else {
                    0
                },
        )
    }

    pub fn min(self, rhs: Fix16) -> Fix16 {
        if self.0 <= rhs.0 {
            self
        } else {
            rhs
        }
    }

    pub fn max(self, rhs: Fix16) -> Fix16 {
        if self.0 >= rhs.0 {
            self
        } else {
            rhs
        }
    }

    pub fn clamp(self, low: Fix16, high: Fix16) -> Fix16 {
        self.min(low).max(high)
    }

    pub fn overflowing_add(self, rhs: Fix16) -> (Fix16, bool) {
        let res = self.0.overflowing_add(rhs.0);
        (Fix16(res.0), res.1)
    }

    pub fn overflowing_sub(self, rhs: Fix16) -> (Fix16, bool) {
        let res = self.0.overflowing_sub(rhs.0);
        (Fix16(res.0), res.1)
    }

    pub fn overflowing_mul(self, rhs: Fix16) -> (Fix16, bool) {
        let mut overflowed = false;
        let mut product = self.0 as i64 * rhs.0 as i64;

        let upper = (product >> 47) as u32;
        if product < 0 {
            if !upper != 0 {
                overflowed = true;
            }

            if crate::no_rounding() {
                product -= 1;
            }
        } else if upper != 0 {
            overflowed = true;
        }

        let res = {
            let mut res = (product >> 16) as i32;

            if !crate::no_rounding() {
                res += ((product & 0x8000) >> 15) as i32;
            }

            Fix16(res)
        };

        (res, overflowed)
    }

    pub fn overflowing_div(self, rhs: Fix16) -> (Fix16, bool) {
        if rhs.0 == 0 {
            return (FIX16_MIN, false);
        }

        let mut remainder = self.0.abs() as u32;
        let mut divider = rhs.0.abs() as u32;
        let mut quotient: u32 = 0;
        let mut bit_pos = 17i32;
        let mut overflowed = false;

        // Kick-start the division a bit.
        // This improves speed in the worst-case scenarios where N and D are large
        // It gets a lower estimate for the result by N/(D >> 17 + 1).
        if (divider & 0xFFF0_0000) != 0 {
            let shifted_div = (divider >> 17) + 1;
            quotient = remainder / shifted_div;
            remainder -= ((quotient as u64 * divider as u64) >> 17) as u32;
        }

        // If the divider is divisible by 2^n, take advantage of it.
        while (divider & 0xF) == 0 && bit_pos >= 4 {
            divider >>= 4;
            bit_pos -= 4;
        }

        while remainder != 0 && bit_pos >= 0 {
            let shift = remainder.leading_zeros().min(bit_pos as u32);

            remainder <<= shift;
            bit_pos -= shift as i32;

            let div = remainder / divider;
            remainder = remainder % divider;
            quotient += div << bit_pos;

            overflowed |= div & !(u32::MAX >> bit_pos) != 0;

            remainder <<= 1;
            bit_pos -= 1;
        }

        if !crate::no_rounding() {
            quotient += 1;
        }

        let res = quotient as i32 >> 1;

        if ((self.0 as u32 ^ rhs.0 as u32) & 0x80000000) != 0 {
            overflowed |= res == FIX16_MIN.0;

            (Fix16(-res), overflowed)
        } else {
            (Fix16(res), overflowed)
        }
    }

    pub fn saturating_add(self, rhs: Fix16) -> Fix16 {
        let res = self.0.overflowing_add(rhs.0);
        if res.1 {
            if self.0 >= 0 {
                FIX16_MAX
            } else {
                FIX16_MIN
            }
        } else {
            Fix16(res.0)
        }
    }

    pub fn saturating_sub(self, rhs: Fix16) -> Fix16 {
        let res = self.0.overflowing_sub(rhs.0);
        if res.1 {
            if self.0 >= 0 {
                FIX16_MAX
            } else {
                FIX16_MIN
            }
        } else {
            Fix16(res.0)
        }
    }

    pub fn saturating_mul(self, rhs: Fix16) -> Fix16 {
        let res = self.overflowing_mul(rhs);

        if res.1 {
            if self.0.is_positive() == rhs.0.is_positive() {
                FIX16_MAX
            } else {
                FIX16_MIN
            }
        } else {
            res.0
        }
    }

    pub fn saturating_div(self, rhs: Fix16) -> Fix16 {
        let res = self.overflowing_div(rhs);

        if res.1 {
            if self.0.is_positive() == rhs.0.is_positive() {
                FIX16_MAX
            } else {
                FIX16_MIN
            }
        } else {
            res.0
        }
    }

    pub fn from_hex_str(str: &String) -> Result<Fix16, String> {
        let chars = str.chars().collect::<Vec<char>>();
        let negative = chars[0] == '-';
        let start_offset = {
            let off_one = if negative { 1 } else { 0 };
            let off_two = if chars.len() > off_one
                && chars[off_one] == '0'
                && chars.len() > off_one + 1
                && (chars[off_one + 1] == 'x' || chars[off_one + 1] == 'X')
            {
                2
            } else {
                0
            };
            off_one + off_two
        };

        fn is_num_str(inp: &String) -> bool {
            let chars = inp.chars().collect::<Vec<char>>();
            for x in 0..inp.len() {
                if !(chars[x] >= '0' && chars[x] <= '9') {
                    return false;
                }
            }
            true
        }

        let contains_dec = str.find('.');

        let out_val: u32;

        if let Some(dec_point_offset) = contains_dec {
            let pre_dec_str = chars[start_offset..dec_point_offset]
                .iter()
                .collect::<String>();
            let mut post_dec_str = chars[dec_point_offset + 1..chars.len()]
                .iter()
                .collect::<String>();
            let post_dec_str_len = post_dec_str.len();
            if post_dec_str_len < 4 && is_num_str(&post_dec_str) {
                if post_dec_str_len == 1 {
                    post_dec_str = format!("0{}00", post_dec_str);
                } else if post_dec_str_len == 2 {
                    post_dec_str = format!("{}00", post_dec_str);
                } else {
                    post_dec_str = format!("{}0", post_dec_str);
                }
            }

            let pre_dec_parsed = u16::from_str_radix(&pre_dec_str, 16);
            let post_dec_parsed = u16::from_str_radix(&post_dec_str, 16);

            if let Err(why) = pre_dec_parsed {
                return Err(why.to_string());
            } else if let Err(why) = post_dec_parsed {
                return Err(why.to_string());
            }

            out_val = (pre_dec_parsed.unwrap() as u32) << 16 | (post_dec_parsed.unwrap() as u32);
        } else {
            let pre_dec_str = chars[start_offset..chars.len()].iter().collect::<String>();
            let pre_dec_parsed = u16::from_str_radix(&pre_dec_str, 16);

            if let Err(why) = pre_dec_parsed {
                return Err(why.to_string());
            }

            out_val = (pre_dec_parsed.unwrap() as u32) << 16;
        }

        if negative {
            Ok(Fix16((out_val as i32) * -1))
        } else {
            Ok(Fix16(out_val as i32))
        }
    }

    pub fn from_str(str: &String) -> Result<Fix16, String> {
        match str.parse::<f32>() {
            Ok(c) => Ok(Fix16::from(c)),
            Err(why) => Err(why.to_string()),
        }
    }

    pub fn sqrt(self) -> Fix16 {
        let mut num = if self.0.is_negative() {
            -self.0
        } else {
            self.0
        } as u32;
        let mut res = 0u32;
        let mut bit;

        if (num & 0xFFF00000) != 0 {
            bit = 1u32 << 30;
        } else {
            bit = 1u32 << 18;
        }

        while bit > num {
            bit >>= 2;
        }

        for x in 0..2 {
            while bit != 0 {
                if num >= res + bit {
                    num -= res + bit;
                    res = (res >> 1) + bit;
                } else {
                    res >>= 1;
                }
                bit >>= 2;
            }

            if x == 0 {
                if num > std::u16::MAX as u32 {
                    num -= res;
                    num = (num << 16) - 0x8000;
                    res = (res << 16) + 0x8000;
                } else {
                    num <<= 16;
                    res <<= 16;
                }

                bit = 1 << 14;
            }
        }

        if !crate::no_rounding() && num > res {
            res += 1;
        }

        if self.0.is_negative() {
            Fix16((res as i32) * -1)
        } else {
            Fix16(res as i32)
        }
    }

    pub fn sin(self) -> Fix16 {
        let shl_pi = Fix16(FIX16_PI.0 << 1);
        let shr_pi = Fix16(FIX16_PI.0 >> 1);
        let mut temp_angle = self % shl_pi;

        if temp_angle.0 < 0 {
            temp_angle += shl_pi;
        }
        
        let out_val;
        if temp_angle.0 >= FIX16_PI.0 {
            temp_angle -= FIX16_PI;
            if temp_angle.0 >= shr_pi.0 {
                temp_angle = FIX16_PI - temp_angle;
            }

            out_val = -(if temp_angle.0 >= crate::consts::F16_SIN_LUT_COUNT { FIX16_ONE.0 } else { crate::consts::F16_SIN_LUT[temp_angle.0 as usize] as i32 });
        } else {
            if temp_angle.0 >= shr_pi.0 {
                temp_angle = FIX16_PI - temp_angle;
            }
            
            out_val = if temp_angle.0 >= crate::consts::F16_SIN_LUT_COUNT { FIX16_ONE.0 } else { crate::consts::F16_SIN_LUT[temp_angle.0 as usize] as i32 };
        }

        Fix16(out_val)
    }

    pub fn cos(self) -> Fix16 {
        Fix16(self.0 + (FIX16_PI.0 >> 1)).sin()
    }

    pub fn tan(self) -> Fix16 {
        self.sin().saturating_div(self.cos())
    }
}

impl ops::Add for Fix16 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Fix16(self.0 + rhs.0)
    }
}

impl ops::AddAssign for Fix16 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl ops::Sub for Fix16 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Fix16(self.0 - rhs.0)
    }
}

impl ops::SubAssign for Fix16 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl ops::Mul for Fix16 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut product = self.0 as i64 * rhs.0 as i64;
        let upper = (product >> 47) as u32;

        if product < 0 {
            if !upper != 0 {
                return FIX16_OVERFLOW;
            }

            if !crate::no_rounding() {
                product -= 1;
            }
        } else if upper != 0 {
            return FIX16_OVERFLOW;
        }

        if crate::no_rounding() {
            Fix16((product >> 16) as i32)
        } else {
            Fix16(((product >> 16) + ((product & 0x8000) >> 15)) as i32)
        }
    }
}

impl ops::MulAssign for Fix16 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl ops::Div for Fix16 {
    type Output = Self;

    // This uses a hardware 32/32 bit division multiple times, until we have
    // computed all the bits in (a<<17)/b. Usually this takes 1-3 iterations.
    fn div(self, rhs: Self) -> Self::Output {
        if rhs.0 == 0 {
            return FIX16_MIN;
        }

        let mut remainder = self.0.abs() as u32;
        let mut divider = rhs.0.abs() as u32;
        let mut quotient: u32 = 0;
        let mut bit_pos = 17i32;

        // Kick-start the division a bit.
        // This improves speed in the worst-case scenarios where N and D are large
        // It gets a lower estimate for the result by N/(D >> 17 + 1).
        if (divider & 0xFFF0_0000) != 0 {
            let shifted_div = (divider >> 17) + 1;
            quotient = remainder / shifted_div;
            remainder -= ((quotient as u64 * divider as u64) >> 17) as u32;
        }

        // If the divider is divisible by 2^n, take advantage of it.
        while (divider & 0xF) == 0 && bit_pos >= 4 {
            divider >>= 4;
            bit_pos -= 4;
        }

        while remainder != 0 && bit_pos >= 0 {
            let shift = remainder.leading_zeros().min(bit_pos as u32);

            remainder <<= shift;
            bit_pos -= shift as i32;

            let div = remainder / divider;
            remainder = remainder % divider;
            quotient += div << bit_pos;

            remainder <<= 1;
            bit_pos -= 1;
        }

        if !crate::no_rounding() {
            quotient += 1;
        }

        if ((self.0 as u32 ^ rhs.0 as u32) & 0x80000000) != 0 {
            Fix16(-(quotient as i32 >> 1))
        } else {
            Fix16(quotient as i32 >> 1)
        }
    }
}

impl ops::DivAssign for Fix16 {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl ops::Rem for Fix16 {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Fix16(self.0 % rhs.0)
    }
}

impl ops::RemAssign for Fix16 {
    fn rem_assign(&mut self, rhs: Self) {
        *self = *self % rhs;
    }
}

impl ops::Shl for Fix16 {
    type Output = Self;

    fn shl(self, rhs: Self) -> Self::Output {
        Fix16(self.0 << i32::from(rhs))
    }
}

impl ops::ShlAssign for Fix16 {
    fn shl_assign(&mut self, rhs: Self) {
        *self = *self << rhs;
    }
}

impl ops::Shr for Fix16 {
    type Output = Self;

    fn shr(self, rhs: Self) -> Self::Output {
        Fix16(self.0 >> i32::from(rhs))
    }
}

impl ops::ShrAssign for Fix16 {
    fn shr_assign(&mut self, rhs: Self) {
        *self = *self >> rhs;
    }
}

impl PartialEq for Fix16 {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
