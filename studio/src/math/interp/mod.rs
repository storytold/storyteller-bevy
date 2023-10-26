//! Helpers for interpolating between values.

pub use ecs::*;
pub use traits::*;

mod ecs;
mod traits;

#[derive(Clone, Copy, Debug)]
pub enum Easing {
    Linear,
    Sine,
}

impl Default for Easing {
    fn default() -> Self {
        Self::Linear
    }
}
