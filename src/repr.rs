
use std::convert::TryInto;

use ibig::{IBig, ibig};
use crate::ibig_ext::{log_rem, magnitude};

// FIXME: this should be a enum when enum const is supported in generic argument
/// Defines rounding modes of the floating numbers.
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod RoundingMode {
    /// Round to the nearest value, ties are rounded to an even value. (default mode)
    pub const HalfEven: u8 = 0;

    /// Round toward +infinity
    pub const Up: u8 = 1;

    /// Round toward -infinity
    pub const Down: u8 = 2;

    /// Round toward 0
    pub const Zero: u8 = 3;

    /// Round to the nearest value, ties away from zero
    pub const HalfAway: u8 = 4;
}

/// An arbitrary precision floating number represented as `mantissa * radix^scale`
/// mantissa < radix^precision. The representation is normalized when mantissa is coprime to radix.
///
/// The const generic parameters will be abbreviated as Radix -> E, Rounding -> R.
/// Radix should be in range \[2, isize::MAX\], and Rounding value has to be one of [RoundingMode]
#[allow(non_upper_case_globals)]
#[derive(Clone, Debug)]
pub struct FloatRepr<const Radix: usize, const Rounding: u8> {
    pub(crate) mantissa: IBig,
    pub(crate) exponent: isize,
    pub(crate) precision: usize,
}

impl<const E: usize, const R: u8> FloatRepr<E, R> {
    #[inline]
    pub fn precision(&self) -> usize {
        self.precision
    }

    /// Get the integer k such that `radix^(k-1) <= value < radix^k`.
    /// If value is 0, then `k = 0` is returned.
    #[inline]
    pub(crate) fn actual_precision(value: &IBig) -> usize {
        if value == &ibig!(0) {
            return 0
        };

        let (e, _) = log_rem(&magnitude(value), E);
        let e: usize = e.try_into().unwrap();
        e + 1
    }
}

/// Multi-precision float number with binary exponent
#[allow(non_upper_case_globals)]
pub type BinaryRepr<const Rounding: u8> = FloatRepr<2, Rounding>;
/// Multi-precision decimal number with decimal exponent
#[allow(non_upper_case_globals)]
pub type DecimalRepr<const Rounding: u8> = FloatRepr<10, Rounding>;
