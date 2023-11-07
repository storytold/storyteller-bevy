use bevy::prelude::Vec3;

mod float_comparison;
pub mod geo;
mod internal;
pub mod interp;
pub mod matrix;

pub use float_comparison::*;
pub use geo::*;
pub use interp::*;
pub use matrix::*;

pub trait IsNonUniform {
    fn is_non_uniform(&self) -> bool;
}

impl IsNonUniform for Vec3 {
    fn is_non_uniform(&self) -> bool {
        !self.x.nearly_eq_within(self.y, 1e-3) || !self.x.nearly_eq_within(self.z, 1e-3)
    }
}
