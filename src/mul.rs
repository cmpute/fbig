use core::ops::Mul;
use crate::{repr::FloatRepr, utils::mul_hi};

impl<const E: usize, const R: u8> Mul for &FloatRepr<E, R> {
    type Output = FloatRepr<E, R>;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        let precision = self.precision.max(rhs.precision);
        let mantissa = mul_hi::<E>(&self.mantissa, &rhs.mantissa, precision + 1);
        let exponent = self.exponent + rhs.exponent;
        FloatRepr { mantissa, exponent, precision: precision + 1 }.with_precision(precision)
    }
}

impl<const E: usize, const R: u8> Mul for FloatRepr<E, R> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        (&self).mul(&rhs)
    }
}
impl<const E: usize, const R: u8> Mul<FloatRepr<E, R>> for &FloatRepr<E, R> {
    type Output = FloatRepr<E, R>;
    #[inline]
    fn mul(self, rhs: FloatRepr<E, R>) -> Self::Output {
        self.mul(&rhs)
    }
}
