use bevy::prelude::*;

pub mod gizmos;
pub mod selection;
pub mod transformation;

pub use selection::*;
pub use transformation::*;

pub struct InteractionPlugin;
impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((SelectionPlugin, TransformationPlugin));
    }
}
