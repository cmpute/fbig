use core::ops::{Add, Sub};
use ibig::UBig;
use crate::utils::{shl_radix, shr_radix};

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
        let rhs_prec = Self::actual_precision(&rhs.mantissa);
        if ediff + rhs_prec > desire_prec {
            debug_assert!(rhs_prec <= desire_prec);
            let shift = desire_prec - rhs_prec;
            shl_radix::<E>(&mut lhs.mantissa, shift);
            shr_radix::<E>(&mut rhs.mantissa, ediff - shift);
            rhs.exponent -= (ediff - shift) as isize;
        } else {
            shl_radix::<E>(&mut lhs.mantissa, ediff);
        }

        // actuall adding
        let mantissa = lhs.mantissa + rhs.mantissa;
        let exponent = rhs.exponent;
        Self::from_parts_with_precision(mantissa, exponent, desire_prec)
    }
}

impl<const E: usize, const R: u8> Sub for FloatRepr<E, R> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        return self.add(-rhs);
    }
}
