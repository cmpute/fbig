use core::ops::{Add, Sub};
use crate::utils::{shl_radix, shr_radix, get_precision};

use crate::repr::FloatRepr;

impl<const E: usize, const R: u8> Add for FloatRepr<E, R> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        // put the oprand of lower exponent on the left
        let (mut lhs, mut rhs) = if self.exponent < rhs.exponent {
            (self, rhs)
        } else {
            (rhs, self)
        };
        
        // shortcut if lhs is too small
        let ediff = (rhs.exponent - lhs.exponent) as usize;
        let desire_prec = lhs.precision.max(rhs.precision) + 1; // one extra precision for rounding
        if ediff > desire_prec {
            return rhs;
        }

        // align the exponent
        let rhs_prec = lhs.actual_precision();
        let exponent = if ediff + rhs_prec > desire_prec {
            debug_assert!(rhs_prec <= desire_prec);
            let shift = desire_prec - rhs_prec;
            shr_radix::<E>(&mut lhs.mantissa, shift);
            shl_radix::<E>(&mut rhs.mantissa, ediff - shift);
            rhs.exponent - (ediff - shift) as isize
        } else {
            shr_radix::<E>(&mut lhs.mantissa, ediff);
            rhs.exponent
        };

        // actuall adding
        let mantissa = lhs.mantissa + rhs.mantissa;
        Self::from_parts_with_precision(mantissa, exponent, desire_prec - 1)
    }
}

impl<const E: usize, const R: u8> Sub for FloatRepr<E, R> {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        return self.add(-rhs);
    }
}

// TODO: carefully determine whether the opperations take reference or value
impl<const E: usize, const R: u8> Add for &FloatRepr<E, R> {
    type Output = FloatRepr<E, R>;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        self.clone().add(rhs.clone())
    }
}
impl<const E: usize, const R: u8> Sub for &FloatRepr<E, R> {
    type Output = FloatRepr<E, R>;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.add(&(-rhs))
    }
}
impl<const E: usize, const R: u8> Sub<FloatRepr<E, R>> for &FloatRepr<E, R> {
    type Output = FloatRepr<E, R>;
    #[inline]
    fn sub(self, rhs: FloatRepr<E, R>) -> Self::Output {
        self.add(&(-rhs))
    }
}

