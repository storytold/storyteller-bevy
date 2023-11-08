//! Helper traits for floating-point equality checks with configurable tolerance.

use bevy::{math::DVec3, prelude::Vec3};

use super::internal::Zero;

pub trait NearlyEq<Rhs, Tolerance = Rhs> {
    const DEFAULT_TOLERANCE: Tolerance;

    #[inline]
    fn nearly_eq(self, rhs: Rhs) -> bool
    where
        Self: Sized,
    {
        self.nearly_eq_within(rhs, Self::DEFAULT_TOLERANCE)
    }

    fn nearly_eq_within(self, rhs: Rhs, tolerance: Tolerance) -> bool;
}

impl NearlyEq<f32> for f32 {
    const DEFAULT_TOLERANCE: f32 = f32::EPSILON;

    #[inline]
    fn nearly_eq_within(self, rhs: f32, tolerance: f32) -> bool {
        (self - rhs).abs() < tolerance
    }
}

impl NearlyEq<f64> for f64 {
    const DEFAULT_TOLERANCE: f64 = f64::EPSILON;

    #[inline]
    fn nearly_eq_within(self, rhs: f64, tolerance: f64) -> bool {
        (self - rhs).abs() < tolerance
    }
}

impl NearlyEq<Vec3, f32> for Vec3 {
    const DEFAULT_TOLERANCE: f32 = <f32 as NearlyEq<f32>>::DEFAULT_TOLERANCE;

    fn nearly_eq_within(self, rhs: Vec3, tolerance: f32) -> bool {
        self.x.nearly_eq_within(rhs.x, tolerance)
            && self.y.nearly_eq_within(rhs.y, tolerance)
            && self.z.nearly_eq_within(rhs.z, tolerance)
    }
}

impl Zero for Vec3 {
    #[inline]
    fn zero() -> Self {
        Vec3::ZERO
    }
}

impl NearlyEq<DVec3, f64> for DVec3 {
    const DEFAULT_TOLERANCE: f64 = <f64 as NearlyEq<f64>>::DEFAULT_TOLERANCE;

    fn nearly_eq_within(self, rhs: DVec3, tolerance: f64) -> bool {
        self.x.nearly_eq_within(rhs.x, tolerance)
            && self.y.nearly_eq_within(rhs.y, tolerance)
            && self.z.nearly_eq_within(rhs.z, tolerance)
    }
}

impl Zero for DVec3 {
    #[inline]
    fn zero() -> Self {
        DVec3::ZERO
    }
}

pub trait NearlyZero<Tolerance = Self> {
    const DEFAULT_TOLERANCE: Tolerance;

    #[inline]
    fn nearly_zero(self) -> bool
    where
        Self: Sized,
    {
        self.nearly_zero_within(Self::DEFAULT_TOLERANCE)
    }

    fn nearly_zero_within(self, tolerance: Tolerance) -> bool;
}

impl<T, Tolerance> NearlyZero<Tolerance> for T
where
    T: Zero,
    T: NearlyEq<T, Tolerance>,
{
    const DEFAULT_TOLERANCE: Tolerance = <Self as NearlyEq<T, Tolerance>>::DEFAULT_TOLERANCE;

    #[inline]
    fn nearly_zero_within(self, tolerance: Tolerance) -> bool {
        self.nearly_eq_within(<Self as Zero>::zero(), tolerance)
    }
}
