use std::{ops, u32};

use ops::Sub;

use crate::no_rounding;

#[derive(Copy, Clone, Debug)]
pub struct Fix16(pub i32);

const FOUR_DIV_PI: Fix16 = Fix16(0x145F3);
const NEG_FOUR_DIV_PI: Fix16 = Fix16(-0x67c0);
const X4_CORRECTION_COMPONENT: Fix16 = Fix16(0x399A);
const PI_DIV_4: Fix16 = Fix16(0x0000C90F);
const THREE_PI_DIV_4: Fix16 = Fix16(0x00025B2F);

const FIX16_MAX: Fix16 = Fix16(0x7FFFFFFF);
const FIX16_MIN: Fix16 = Fix16(-0x80000000);
const FIX16_OVERFLOW: Fix16 = FIX16_MIN;

const FIX16_PI: Fix16 = Fix16(205887);
const FIX16_E: Fix16 = Fix16(178145);
const FIX16_ONE: Fix16 = Fix16(0x00010000);

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
        Fix16((self.0 & -0x10000) + if (self.0 & 0xFFFF) != 0 { FIX16_ONE.0 } else { 0 })
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

    pub fn overflowing_div(self, rhs: Fix16) -> (Fix16, bool) {
        if rhs.0 == 0 {
            return (FIX16_MIN, false);
        }

        let mut remainder = self.0.abs() as u32;
        let mut divider = rhs.0.abs() as u32;
        let mut quotient: u32 = 0;
        let mut bit_pos = 17;
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
            let shift = remainder.leading_zeros().min(bit_pos);

            remainder <<= shift;
            bit_pos -= shift;

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
}

impl ops::Add for Fix16 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Fix16(self.0 + rhs.0)
    }
}

impl ops::Sub for Fix16 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Fix16(self.0 - rhs.0)
    }
}

impl ops::Mul for Fix16 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut product = self.0 as i64 * rhs.0 as i64;

        if product < 0 {
            
            if !crate::no_rounding() {
                product -= 1;
            }
        }

        if crate::no_rounding() {
            Fix16((product >> 16) as i32)
        } else {
            Fix16(((product >> 16) + ((product & 0x8000) >> 15)) as i32)
        }
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

impl ops::Rem for Fix16 {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Fix16(self.0 % rhs.0)
    }
}