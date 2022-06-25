use core::cmp::Ordering;

use ibig::{IBig, UBig, ibig, ubig, ops::DivRem};
use crate::repr::RoundingMode;

/// "Left shifting" in given radix, i.e. multiply by a power of radix
#[inline]
pub fn shl_radix<const E: usize>(value: &mut IBig, exp: usize) {
    if exp != 0 {
        match E {
            2 => *value <<= exp,
            10 => {
                *value *= IBig::from(5).pow(exp);
                *value <<= exp;
            }
            16 => *value <<= 4 * exp,
            _ => *value *= IBig::from(E).pow(exp)
        }
    }
}

/// "Right shifting" in given radix, i.e. divide by a power of radix
#[inline]
pub fn shr_radix<const E: usize>(value: &mut IBig, exp: usize) {
    if exp != 0 {
        match E {
            2 => *value >>= exp,
            10 => {
                *value >>= exp;
                *value /= ibig!(5).pow(exp);
            }
            16 => *value >>= 4 * exp,
            _ => *value /= IBig::from(E).pow(exp)
        }
    }
}

// TODO: remove, rename or make a trait or upstream this? (check whether this is in GMP)
/// "Right shifting" in given radix, i.e. divide by a power of radix
#[inline]
pub fn ushr_radix<const E: usize>(value: &mut UBig, exp: usize) {
    if exp != 0 {
        match E {
            2 => *value >>= exp,
            10 => {
                *value >>= exp;
                *value /= ubig!(5).pow(exp);
            }
            16 => *value >>= 4 * exp,
            _ => *value /= UBig::from(E).pow(exp)
        }
    }
}

/// "Right shifting" in given radix, i.e. divide by a power of radix.
/// It returns the "shifted" value and the "remainder" part of integer that got removed
#[inline]
pub fn shr_rem_radix<const E: usize>(value: &IBig, exp: usize) -> (IBig, IBig) {
    if exp != 0 {
        if E == 2 {
            let rem = value & ((ibig!(1) << exp) - 1u8);
            (value >> exp, rem)
        } else {
            value.div_rem(IBig::from(E).pow(exp))
        }
    } else {
        (value.clone(), ibig!(0))
    }
}

/// Return the rounding bit based on the remainder (mod Radix)
#[inline]
pub fn round_with_rem<const E: usize, const R: u8>(mantissa: &mut IBig, rem: isize) {
    match (R, rem.signum()) {
        (_, 0) => {},
        (RoundingMode::Zero, _) => {},
        (RoundingMode::Down, 1) => {},
        (RoundingMode::Down, -1) => *mantissa -= 1u8,
        (RoundingMode::Up, 1) => *mantissa += 1u8,
        (RoundingMode::Up, -1) => {},
        (RoundingMode::HalfEven | RoundingMode::HalfAway, _) => {
            let double = (rem as usize) * 2;
            match E.cmp(&double) {
                Ordering::Greater => if rem > 0 {
                    *mantissa += 1u8
                },
                Ordering::Equal => match R {
                    RoundingMode::HalfEven => {
                        // ties to even
                        if &*mantissa % 2i8 != 0 {
                            *mantissa += 1u8;
                        }
                    },
                    RoundingMode::HalfAway => {
                        // ties away from zero
                        if rem > 0 {
                            *mantissa += 1u8;
                        } else {
                            *mantissa -= 1u8;
                        }
                    },
                    _ => unreachable!()
                },
                Ordering::Less => if rem < 0 {
                    *mantissa -= 1u8
                }
            };
        },
        _ => unreachable!()
    }
}
