use std::ops;

use bevy::prelude::Vec3;

use crate::math::{
    internal::{Clamp, One, Zero},
    NearlyZero,
};

pub trait Lerp<Operand> {
    /// Interpolate linearly between `a` and `b`, using `self` as the `t` parameter.
    fn lerp(self, a: Operand, b: Operand) -> Operand;
}

pub trait InvLerp<T: Sized> {
    /// Derive a `t` interpolation parameter for `self`, where `a` is the
    /// "origin" and `b` is the "destination".
    ///
    /// # Examples
    ///
    /// ```
    /// use studio::math::{NearlyEq, InvLerp};
    ///
    /// let t = 48_f32.inv_lerp(32., 64.);
    /// // The expected result is 0.5, because 48 is halfway between 32 and 64.
    /// assert!(t.nearly_eq(0.5));
    /// ```
    fn inv_lerp(self, a: Self, b: Self) -> T;
}

pub trait Remap<T: Sized>
where
    Self: Sized,
{
    /// Re-map a value from its interpolated position in the range `from`, to
    /// the equivalent position in the range `to`.
    ///
    /// # Examples
    ///
    /// ```
    /// use studio::math::{NearlyEq, Remap};
    ///
    /// let x = 48_f32.remap((32., 64.), (32., 128.));
    /// // The expected result is 80, because 48 is halfway between 32 and 64,
    /// // and 80 is halfway between 32 and 128.
    /// assert!(x.nearly_eq_within(80., 1e-3));
    /// ```
    fn remap(self, from: (Self, Self), to: (Self, Self)) -> Self;
}

pub trait CurveSin {
    /// Fit a `t` interpolation parameter to a sine curve. Mostly useful for
    /// "easing" between steps in an animation.
    ///
    /// https://www.desmos.com/calculator/k9x4c4cr5y
    ///
    /// # Example
    ///
    /// ```ignore
    /// // This is fine, but it will feel "jerky" because the velocity is
    /// // constant from start to end.
    /// fn update_position(dt: f32, start: Vec3, end: Vec3, t: f32) -> Vec3 {
    ///     (t + dt).clamp(0., 1.).lerp(start, end)
    /// }
    ///
    /// // This will smoothly accelerate from a standstill at t=0, and
    /// // smoothly decelerate back to a stop at t=1.
    /// fn update_position_eased(dt: f32, start: Vec3, end: Vec3, t: f32) -> Vec3 {
    ///     (t + dt)
    ///         .clamp(0., 1.)
    ///         .curve_sin()
    ///         .lerp(start, end)
    /// }
    /// ```
    fn curve_sin(self) -> Self;

    /// Works like `curve_sin`, except that only the lower half of the line is
    /// curved, with the upper half progressing linearly.
    ///
    /// https://www.desmos.com/calculator/ngceukl6g5
    fn curve_sin_in(self) -> Self;

    /// Works like `curve_sin`, except that only the upper half of the line is
    /// curved, with the lower half progressing linearly.
    ///
    /// https://www.desmos.com/calculator/umvvxhkd46
    fn curve_sin_out(self) -> Self;
}

impl<T, Operand> Lerp<Operand> for T
where
    T: Copy,
    T: One,
    T: ops::Sub<Output = T>,
    Operand: ops::Mul<T, Output = Operand>,
    Operand: ops::Add<Output = Operand>,
{
    #[inline]
    fn lerp(self, a: Operand, b: Operand) -> Operand {
        a * (Self::one() - self) + b * self
    }
}

impl<T> InvLerp<T> for T
where
    T: One + Zero,
    T: Clamp,
    T: NearlyZero,
    T: Copy,
    T: ops::Sub<Output = T>,
    T: ops::Div<Output = T>,
{
    #[inline]
    fn inv_lerp(self, a: T, b: T) -> T {
        let dividend = b - a;
        if !dividend.nearly_zero() {
            ((self - a) / dividend).clamp(T::zero(), T::one())
        } else {
            T::one()
        }
    }
}

// Vector inverse lerp requires a special implementation, yoinked from here:
// https://discussions.unity.com/t/inverselerp-for-vector3/177038
impl InvLerp<f32> for Vec3 {
    #[inline]
    fn inv_lerp(self, a: Vec3, b: Vec3) -> f32 {
        let ab = b - a;
        let av = self - a;
        if !ab.nearly_zero() {
            av.dot(ab) / ab.length_squared()
        } else {
            1.
        }
    }
}

impl<T, Operand> Remap<T> for Operand
where
    Operand: InvLerp<T>,
    T: Lerp<Operand>,
{
    #[inline]
    fn remap(self, from: (Self, Self), to: (Self, Self)) -> Self {
        self.inv_lerp(from.0, from.1).lerp(to.0, to.1)
    }
}

impl CurveSin for f32 {
    fn curve_sin(self) -> Self {
        use std::f32::consts::PI;

        -1. * ((self * PI).cos() / 2.) + 0.5
    }

    fn curve_sin_in(self) -> Self {
        use std::f32::consts::PI;

        1. - ((self * PI) / 2.).cos()
    }

    fn curve_sin_out(self) -> Self {
        use std::f32::consts::PI;

        ((self * PI) / 2.).sin()
    }
}

impl CurveSin for f64 {
    fn curve_sin(self) -> Self {
        use std::f64::consts::PI;

        -1. * ((self * PI).cos() / 2.) + 0.5
    }

    fn curve_sin_in(self) -> Self {
        use std::f64::consts::PI;

        1. - ((self * PI) / 2.).cos()
    }

    fn curve_sin_out(self) -> Self {
        use std::f64::consts::PI;

        ((self * PI) / 2.).sin()
    }
}
