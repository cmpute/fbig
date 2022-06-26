
use ibig::IBig;
use crate::utils::get_precision;

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
/// mantissa < radix^precision. The representation is always normalized (mantissa is not divisible by radix).
///
/// The const generic parameters will be abbreviated as Radix -> E, Rounding -> R.
/// Radix should be in range \[2, isize::MAX\], and Rounding value has to be one of [RoundingMode]
#[allow(non_upper_case_globals)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FloatRepr<const Radix: usize, const Rounding: u8> {
    pub(crate) mantissa: IBig,
    pub(crate) exponent: isize,
    pub(crate) precision: usize,
}

impl<const E: usize, const R: u8> FloatRepr<E, R> {
    /// Get the maximum precision set for the float number.
    #[inline]
    pub fn precision(&self) -> usize {
        self.precision
    }

    /// Get the actual precision needed for the float number.
    /// 
    /// Shrink to this value using [Self::with_precision] will not cause loss of float precision.
    #[inline]
    pub fn actual_precision(&self) -> usize {
        get_precision::<E>(&self.mantissa)
    }

    fn ceil(&self) -> Self {
        unimplemented!()
    }

    fn floor(&self) -> Self {
        unimplemented!()
    }

    fn trunc(&self) -> Self {
        unimplemented!()
    }

    fn fract(&self) -> Self {
        unimplemented!()
    }
}

/// Multi-precision float number with binary exponent
#[allow(non_upper_case_globals)]
pub type BinaryRepr<const Rounding: u8> = FloatRepr<2, Rounding>;
/// Multi-precision decimal number with decimal exponent
#[allow(non_upper_case_globals)]
pub type DecimalRepr<const Rounding: u8> = FloatRepr<10, Rounding>;
