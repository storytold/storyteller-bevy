use bevy::prelude::*;

#[cfg(feature = "wasm")]
mod wasm;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "wasm")]
        app.add_systems(
            PostUpdate,
            wasm::notify_entity_spawn_changes.map(bevy::utils::error),
        );
    }
}
