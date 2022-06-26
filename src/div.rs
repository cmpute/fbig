use core::ops::Div;
use std::convert::TryInto;
use ibig::{IBig, ibig, ops::Abs};
use crate::{repr::FloatRepr, utils::{shr_rem_radix, get_precision}, ibig_ext::log_rem};

impl<const E: usize, const R: u8> FloatRepr<E, R> {
    /// Calculate the multiplicative inverse (1/x)
    pub fn recip(&self) -> Self {
        // FIXME: specialize the case of binary float
        // FIXME: normalize self to close to 1 for better convergence

        // first estimate the reciprocal using the first several digits.
        // 1. find high parts of mantissa that roughly fits in i32
        // 2. calculate the highest power of radix that fits in i64 (this should be constant)
        // 3. use the power divided by the high parts of mantissa as a fixed point estimation
        let (shifts, rem) = log_rem(&(i64::MAX as u64).into(), E); // FIXME: make this step constant
        let rem: u64 = rem.try_into().unwrap();
        let pow = i64::MAX - rem as i64; // highest possible power of radix
        let mantissa_digits = self.actual_precision();
        if shifts > self.precision + mantissa_digits {
            // if the quotient already has the desired precision, just use the quotient
            let est = IBig::from(pow) / &self.mantissa;
            let est_digits = get_precision::<E>(&est);
            let result = Self {
                mantissa: est,
                exponent: -self.exponent - shifts as isize,
                precision: est_digits
            }.with_precision(self.precision);
            return result;
        }

        let mut guess = if mantissa_digits > shifts / 2 {
            let (mantissa_hi, _) = shr_rem_radix::<E>(&self.mantissa, mantissa_digits - shifts / 2);
            let mantissa_hi: i64 = mantissa_hi.abs().try_into().unwrap();
            let est = pow / mantissa_hi;
            Self {
                mantissa: IBig::from(est),
                exponent: -self.exponent - ((shifts + 1) / 2) as isize,
                precision: self.precision + 1 // one more digit for rounding
            }
        } else {
            let mantissa: i64 = self.mantissa.clone().try_into().unwrap();
            let est = pow / mantissa;
            Self {
                mantissa: IBig::from(est),
                exponent: -self.exponent - shifts as isize,
                precision: self.precision + 1
            }
        };

        // then perform newton interations x = x(2 - a*x)
        let two = Self::from_integer(ibig!(2), self.precision + 1);
        loop {
            let new_guess = &guess * (&two - self * &guess);
            println!("new_guess: {}, prec: {}", new_guess.to_decimal(), new_guess.precision());
            if new_guess == guess {
                break new_guess;
            }
            guess = new_guess;
        }
    }
}

impl<const E: usize, const R: u8> Div for FloatRepr<E, R> {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        // TODO: directly divide, refactor with from_ratio
        self * rhs.recip()
    }
}
