use core::cmp::Ordering;
use core::convert::TryInto;

use ibig::{IBig, UBig, ibig, ubig, ops::DivRem};
use crate::{repr::RoundingMode, ibig_ext::{log, magnitude}};

/// Get the integer k such that `radix^(k-1) <= value < radix^k`.
/// If value is 0, then `k = 0` is returned.
pub fn get_precision<const E: usize>(value: &IBig) -> usize{
    if value == &ibig!(0) {
        return 0
    };

    let e = log(&magnitude(value), E);
    let e: usize = e.try_into().unwrap();
    e + 1
}

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

/// Calculate the high parts of a * b.
/// 
/// It's equivalent to find `a * b / E^c` such that it's in the range `[E^(prec-1), E^prec)`
#[inline]
pub fn mul_hi<const E: usize>(a: &IBig, b: &IBig, prec: usize) -> IBig {
    let mut c = a * b;
    let prec_actual = get_precision::<E>(&c);
    if prec_actual > prec {
        shr_radix::<E>(&mut c, prec_actual - prec);
    }
    c
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
        match E {
            2 => {
                // FIXME: a dedicate method to extract low bits for IBig might be helpful here
                let rem = value & ((ibig!(1) << exp) - 1u8);
                (value >> exp, rem)
            },
            10 => {
                let rem1 = value & ((ibig!(1) << exp) - 1u8);
                let (q, rem2) = (value >> exp).div_rem(ibig!(5).pow(exp));
                let rem = (rem2 << exp) + rem1;
                (q, rem)
            },
            16 => {
                let rem = value & ((ibig!(1) << (4 * exp)) - 1u8);
                (value >> 4 * exp, rem)
            },
            _ => value.div_rem(IBig::from(E).pow(exp))
        }
    } else {
        (value.clone(), ibig!(0))
    }
}

/// Return the rounding bit based on the remainder (mod Radix)
#[inline]
pub fn round_with_rem<const E: usize, const R: u8>(mantissa: &mut IBig, rem: isize) {
    assert!((rem.abs() as usize) < E);

    match (R, rem.signum()) {
        (_, 0) => {},
        (RoundingMode::Zero, _) => {},
        (RoundingMode::Down, 1) => {},
        (RoundingMode::Down, -1) => *mantissa -= 1u8,
        (RoundingMode::Up, 1) => *mantissa += 1u8,
        (RoundingMode::Up, -1) => {},
        (RoundingMode::HalfEven | RoundingMode::HalfAway, _) => {
            let double = if rem < 0 {
                (rem + E as isize) * 2
            } else {
                rem * 2
            } as usize;
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
