//! Arithmetic utilities.

#![cfg_attr(not(feature = "std"), no_std)]

/// Provides ability to perform ceiling division operations on integers.
pub trait DivCeil: Sized {
    /// Performs ceiling division usign supplied operands.
    fn div_ceil(self, other: Self) -> Self;

    /// Performs checked ceiling division usign supplied operands.
    ///
    /// Returns `None` in case either divider is zero or the calculation overflowed.
    fn checked_div_ceil(self, other: Self) -> Option<Self>;
}

/// Implements `DivCeil` for the specified type which implements `div`/`rem` ops.
#[macro_export]
macro_rules! impl_div_ceil {
    ($type: ident) => {
        impl DivCeil for $type {
            #[allow(unused_comparisons)]
            fn div_ceil(self, other: Self) -> Self {
                let quot = self / other;
                let rem = self % other;

                if (rem > 0 && other > 0) || (rem < 0 && other < 0) {
                    quot + 1
                } else {
                    quot
                }
            }

            #[allow(unused_comparisons)]
            fn checked_div_ceil(self, other: Self) -> Option<Self> {
                let quot = self.checked_div(other)?;
                let rem = self.checked_rem(other)?;

                if (rem > 0 && other > 0) || (rem < 0 && other < 0) {
                    quot.checked_add(1)
                } else {
                    Some(quot)
                }
            }
        }
    };
    ($($type: ident),+) => {
        $($crate::impl_div_ceil! { $type })+
    }
}

impl_div_ceil! { u8, u16, u32, u64, u128, i8, i16, i32, i64, i128 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn div_ceil() {
        assert_eq!(9.div_ceil(2), 5);
        assert_eq!(10.div_ceil(2), 5);
        assert_eq!(11.div_ceil(2), 6);
        assert_eq!(12.div_ceil(2), 6);
        assert_eq!(0.div_ceil(1), 0);
        assert_eq!(1.div_ceil(1), 1);
    }

    #[test]
    fn checked_div_ceil() {
        assert_eq!(9.checked_div_ceil(2), Some(5));
        assert_eq!(10.checked_div_ceil(2), Some(5));
        assert_eq!(11.checked_div_ceil(2), Some(6));
        assert_eq!(12.checked_div_ceil(2), Some(6));
        assert_eq!(0.checked_div_ceil(1), Some(0));
        assert_eq!(1.checked_div_ceil(1), Some(1));
        assert_eq!(1.checked_div_ceil(0), None);
    }

    #[test]
    fn div_ceil_negative() {
        assert_eq!((0).div_ceil(-1), 0);
        assert_eq!((-1).div_ceil(2), 0);
        assert_eq!((-9).div_ceil(2), -4);
        assert_eq!((-10).div_ceil(2), -5);
        assert_eq!((-11).div_ceil(2), -5);
        assert_eq!((-12).div_ceil(2), -6);
        assert_eq!(0.div_ceil(1), 0);
        assert_eq!((-1).div_ceil(1), -1);

        assert_eq!((-1).div_ceil(-2), 1);
        assert_eq!((-9).div_ceil(-2), 5);
        assert_eq!((-10).div_ceil(-2), 5);
        assert_eq!((-11).div_ceil(-2), 6);
        assert_eq!((-12).div_ceil(-2), 6);
        assert_eq!(0.div_ceil(-1), 0);
        assert_eq!((-1).div_ceil(-1), 1);
    }

    #[test]
    fn checked_div_ceil_negative() {
        assert_eq!((0).checked_div_ceil(-1), Some(0));
        assert_eq!((-1).checked_div_ceil(2), Some(0));
        assert_eq!((-9).checked_div_ceil(2), Some(-4));
        assert_eq!((-10).checked_div_ceil(2), Some(-5));
        assert_eq!((-11).checked_div_ceil(2), Some(-5));
        assert_eq!((-12).checked_div_ceil(2), Some(-6));
        assert_eq!(0.checked_div_ceil(1), Some(0));
        assert_eq!(1.checked_div_ceil(0), None);
        assert_eq!((-1).checked_div_ceil(1), Some(-1));

        assert_eq!((-1).checked_div_ceil(-2), Some(1));
        assert_eq!((-9).checked_div_ceil(-2), Some(5));
        assert_eq!((-10).checked_div_ceil(-2), Some(5));
        assert_eq!((-11).checked_div_ceil(-2), Some(6));
        assert_eq!((-12).checked_div_ceil(-2), Some(6));
        assert_eq!(0.checked_div_ceil(-1), Some(0));
        assert_eq!((-1).checked_div_ceil(-0), None);
        assert_eq!((-1).checked_div_ceil(-1), Some(1));
    }
}
