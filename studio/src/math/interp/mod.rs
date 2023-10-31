//! Helpers for interpolating between values.

pub use ecs::*;
pub use traits::*;

mod ecs;
mod traits;

#[derive(Clone, Copy, Debug)]
pub enum Easing {
    Linear,
    Sine,
    SineOut,
    #[allow(dead_code)]
    SineIn,
}

impl Default for Easing {
    fn default() -> Self {
        Self::Linear
    }
}
