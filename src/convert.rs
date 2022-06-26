use std::convert::TryInto;
use ibig::{IBig, ibig, UBig, ops::DivRem};
use crate::{
    repr::{FloatRepr, BinaryRepr, DecimalRepr},
    utils::{shr_radix, round_with_rem, get_precision},
    ibig_ext::{remove_pow, log_rem, log_pow}
};

impl<const R: u8> From<f32> for BinaryRepr<R> {
    fn from(f: f32) -> Self {
        let bits: u32 = f.to_bits();

        let mut exponent: isize = ((bits >> 23) & 0xff) as isize;
        exponent -= 127 + 23; // bias + mantissa shift

        let mantissa = if exponent == 0 {
            (bits & 0x7fffff) << 1
        } else {
            (bits & 0x7fffff) | 0x800000
        } as i32;
        let mantissa = if bits >> 31 == 0 {
            IBig::from(mantissa)
        } else {
            IBig::from(-mantissa)
        };

        Self { mantissa, exponent, precision: 24 }
    }
}

impl<const R: u8> From<f64> for BinaryRepr<R> {
    fn from(f: f64) -> Self {
        let bits: u64 = f.to_bits();

        let mut exponent: isize = ((bits >> 52) & 0x7ff) as isize;
        exponent -= 1023 + 52; // bias + mantissa shift

        let mantissa = if exponent == 0 {
            (bits & 0xfffffffffffff) << 1
        } else {
            (bits & 0xfffffffffffff) | 0x10000000000000
        } as i64;
        let mantissa = if bits >> 63 == 0 {
            IBig::from(mantissa)
        } else {
            IBig::from(-mantissa)
        };

        Self { mantissa, exponent, precision: 53 }
    }
}

impl<const E: usize, const R: u8> FloatRepr<E, R> {
    /// Create a floating number from a integer
    pub fn from_integer(integer: IBig, precision: usize) -> Self {
        Self::from_parts_with_precision(integer, 0, precision)
    }

    /// Create a floating number by dividing two integers with given precision
    pub fn from_ratio(numerator: IBig, denominator: IBig, precision: usize) -> Self {
        // FIXME: investigate whether it's faster to first calculate the inverse of denom, and then multiply
        // FIXME: find a way to use the fast div support from ibig
        let (mut mantissa, mut rem) = numerator.div_rem(&denominator);
        let mut digits = get_precision::<E>(&mantissa);
        let mut exponent = 0;
        if digits < precision {
            while digits < precision && &rem != &ibig!(0) {
                let (d, r) = (rem * E).div_rem(&denominator);
                rem = r;
                mantissa *= E;
                mantissa += d;
                digits += 1;
                exponent -= 1;
            }
        } else {
            shr_radix::<E>(&mut mantissa, digits - precision);
            exponent = (digits - precision) as isize;
        }
        Self::from_parts_with_precision(mantissa, exponent, precision)
    }

    /// Convert the float number to decimal based exponents.
    /// 
    /// It's equivalent to [Self::with_radix::<10>()]
    #[inline]
    pub fn into_decimal(self) -> DecimalRepr<R> {
        self.with_radix::<10>()
    }

    /// Convert the float number to decimal based exponents.
    #[inline]
    pub fn to_decimal(&self) -> DecimalRepr<R> {
        self.clone().with_radix::<10>()
    }

    /// Convert the float number to binary based exponents.
    /// 
    /// It's equivalent to [Self::with_radix::<2>()]
    #[inline]
    pub fn into_binary(self) -> BinaryRepr<R> {
        self.with_radix::<2>()
    }

    /// Convert the float number to decimal based exponents.
    #[inline]
    pub fn to_binary(&self) -> BinaryRepr<R> {
        self.clone().with_radix::<2>()
    }

    /// Explicitly change the precision of the number.
    /// 
    /// If the given precision is less than the previous value,
    /// it will be rounded following the rounding mode specified by the type parameter.
    pub fn with_precision(self, precision: usize) -> Self {
        let mut result = self;

        // shrink if possible
        if result.precision > precision {
            let actual = result.actual_precision();
            if actual > precision {
                let shift = actual - precision;
                shr_radix::<E>(&mut result.mantissa, shift - 1); // left one additional digit for rounding
                let (mut man, rem) = result.mantissa.div_rem(E as isize);
                round_with_rem::<E, R>(&mut man, rem);
                result.mantissa = man;
                result.exponent += shift as isize;
            }
        }

        result.precision = precision;
        return result;
    }

    /// Explicitly change the rounding mode of the number.
    /// 
    /// This operation has no cost.
    #[inline]
    #[allow(non_upper_case_globals)]
    pub fn with_rounding<const NewR: u8>(self) -> FloatRepr<E, {NewR}> {
        FloatRepr { mantissa: self.mantissa, exponent: self.exponent, precision: self.precision }
    }

    /// Explicitly change the radix of the float number.
    /// 
    /// The precision of the result number will be at most equal to the
    /// precision of the original number (numerically), that is
    /// ```new_radix ^ new_precision <= old_radix ^ old_precision```.
    /// If any rounding happens during the conversion, if will follow
    /// the rounding mode specified by the type parameter.
    #[allow(non_upper_case_globals)]
    pub fn with_radix<const NewE: usize>(self) -> FloatRepr<NewE, R> {
        if NewE == E {
            return FloatRepr { mantissa: self.mantissa, exponent: self.exponent, precision: self.precision };
        }
        // FIXME: shortcut if E is a power of NewE

        // Calculate the new precision
        // new_precision = floor_log_radix2(radix1^precision)
        let precision = log_pow(&UBig::from(E), self.precision, NewE);

        // Convert by calculating logarithm
        // FIXME: currently the calculation is done in full precision, could be vastly optimized
        let result = if self.exponent == 0 {
            // direct copy if the exponent is zero
            return FloatRepr { mantissa: self.mantissa, exponent: 0, precision };
        } else if self.exponent > 0 {
            // denote log with base of radix2 as lgr2, then
            // mantissa * radix1 ^ exp1
            // = mantissa * radix2 ^ lgr2(radix1^exp1)
            // = mantissa * (radix2 ^ floor_lgr2(radix1^exp1) + rem_lgr2(radix1^exp1))
            // = mantissa * ratio * (radix2 ^ floor_lgr2(radix1^exp1))
            // where the ratio is
            // 1 + rem_lgr2(radix1^exp1) / (radix2 ^ floor_lgr2(radix1^exp1))
            // = radix1^exp1 / (radix1^exp1 - rem_lgr2(radix1^exp1))

            let precision_ub = UBig::from(E).pow(self.exponent as usize);
            let (log_v, log_r) = log_rem(&precision_ub, NewE);
            let den = IBig::from(&precision_ub - log_r);
            let num = IBig::from(precision_ub) * self.mantissa;
            let mut value = FloatRepr::<NewE, R>::from_ratio(num, den, precision + 1);
            value.exponent += log_v as isize;
            value
        } else {
            // denote log with base of radix2 as lgr2, then
            // mantissa / radix1 ^ exp1
            // = mantissa / radix2 ^ lgr2(radix1^exp1)
            // = mantissa / (radix2 ^ floor_lgr2(radix1^exp1) + rem_lgr2(radix1^exp1))
            // = mantissa (1 / (radix2 ^ floor_lgr2(..)) - rem_lgr2(..) / (radix2 ^ floor_lgr2(..) * (radix2 ^ floor_lgr2(..) + rem_lgr2(..)))
            // = mantissa * ratio * (1 / (radix2 ^ floor_lgr2(radix1^exp1))
            // where the ratio is
            // 1 - rem_lgr2(radix1^exp1) / (radix2 ^ floor_lgr2(radix1^exp1) + rem_lgr2(radix1^exp1))
            // = radix2 ^ floor_lgr2(radix1^exp1) / radix1^exp1

            let precision_ub = UBig::from(E).pow(-self.exponent as usize);
            let (log_v, log_r) = log_rem(&precision_ub, NewE);
            let num = IBig::from(&precision_ub - log_r) * self.mantissa;
            let den = IBig::from(precision_ub);
            let mut value = FloatRepr::<NewE, R>::from_ratio(num, den, precision + 1);
            value.exponent -= log_v as isize;
            value
        };

        result.with_precision(precision)
    }

    #[allow(non_upper_case_globals)]
    fn with_radix_and_precision<const NewE: usize>(self, precision: usize) -> FloatRepr<NewE, R> {
        // approximate power if precision is small
        // calculate more digits if precision is high
        unimplemented!()
    }

    /// Convert raw parts into a float number, the precision will be inferred from mantissa
    /// (the lowest k such that `mantissa < radix^k`)
    /// 
    /// # Panics
    /// If the mantissa is larger than `radix^usize::MAX`
    #[inline]
    pub fn from_parts(mut mantissa: IBig, mut exponent: isize) -> Self {
        // TODO: prevent using this function internally because we enforce normalized representation
        if E == 2 {
            if let Some(shift) = mantissa.trailing_zeros() {
                mantissa >>= shift;
                exponent += shift as isize;
            };
        } else {
            let shift: isize = remove_pow(&mut mantissa, &E.into()).try_into().unwrap();
            exponent += shift;
        }

        let precision = get_precision::<E>(&mantissa);
        Self { mantissa, exponent, precision }
    }

    /// Convert raw parts into a float number, with given precision.
    #[inline]
    pub fn from_parts_with_precision(mantissa: IBig, exponent: isize, precision: usize) -> Self {
        Self::from_parts(mantissa, exponent).with_precision(precision)
    }

    /// Convert the float number into raw (mantissa, exponent) parts
    #[inline]
    pub fn into_parts(self) -> (IBig, isize) {
        (self.mantissa, self.exponent)
    }

    // TODO: let all these to_* functions return `Approximation`

    /// Convert the float number to native [f32] with the given rounding mode.
    fn to_f32(&self) -> f32 {
        unimplemented!()
    }

    /// Convert the float number to native [f64] with the given rounding mode.
    fn to_f64(&self) -> f64 {
        unimplemented!()
    }

    /// Convert the float number to integer with the given rounding mode.
    fn to_int(&self) -> IBig {
        unimplemented!()
    }
}
