use core::ops::Neg;
use ibig::ops::Abs;
use crate::repr::FloatRepr;

impl<const E: usize, const R: u8> Neg for FloatRepr<E, R> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        let mut result = self;
        result.mantissa = -result.mantissa;
        result
    }
}

impl<const E: usize, const R: u8> Abs for FloatRepr<E, R> {
    type Output = Self;
    fn abs(self) -> Self::Output {
        let mut result = self;
        result.mantissa = result.mantissa.abs();
        result
    }
}
