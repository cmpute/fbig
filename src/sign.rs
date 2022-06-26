use core::ops::Neg;
use ibig::ops::Abs;
use crate::repr::FloatRepr;

impl<const E: usize, const R: u8> Neg for FloatRepr<E, R> {
    type Output = Self;
    fn neg(mut self) -> Self::Output {
        self.mantissa = -self.mantissa;
        self
    }
}

impl<const E: usize, const R: u8> Neg for &FloatRepr<E, R> {
    type Output = FloatRepr<E, R>;
    fn neg(self) -> Self::Output {
        self.clone().neg()
    }
}

impl<const E: usize, const R: u8> Abs for FloatRepr<E, R> {
    type Output = Self;
    fn abs(mut self) -> Self::Output {
        self.mantissa = self.mantissa.abs();
        self
    }
}
