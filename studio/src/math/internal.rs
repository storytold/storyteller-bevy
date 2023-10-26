//! Helpers to make it easier to implement traits for both `f32` and `f64`
//! without needing to copy and paste identical implementations.

#[rustfmt::skip] pub(super) trait Zero { fn zero() -> Self; }
#[rustfmt::skip] impl Zero for f32 { #[inline] fn zero() -> Self { 0_f32 } }
#[rustfmt::skip] impl Zero for f64 { #[inline] fn zero() -> Self { 0_f64 } }

#[rustfmt::skip] pub(super) trait One { fn one() -> Self; }
#[rustfmt::skip] impl One for f32 { #[inline] fn one() -> Self { 1_f32 } }
#[rustfmt::skip] impl One for f64 { #[inline] fn one() -> Self { 1_f64 } }

pub(super) trait Clamp {
    fn clamp(self, min: Self, max: Self) -> Self;
}

impl<T> Clamp for T
where
    T: Sized + Copy + PartialOrd + core::fmt::Debug,
{
    #[inline]
    fn clamp(mut self, min: Self, max: Self) -> Self {
        assert!(
            min <= max,
            "min > max, or either was NaN. min = {min:?}, max = {max:?}"
        );
        if self < min {
            self = min
        }
        if self > max {
            self = max
        }
        self
    }
}
