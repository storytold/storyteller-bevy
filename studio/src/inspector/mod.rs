use bevy::prelude::*;

#[cfg(feature = "wasm")]
mod wasm;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(
        &self,
        #[cfg(feature = "wasm")] app: &mut App,
        #[cfg(not(feature = "wasm"))] _: &mut App,
    ) {
        #[cfg(feature = "wasm")]
        app.add_systems(
            PostUpdate,
            wasm::notify_entity_spawn_changes.map(bevy::utils::error),
        );
    }
}
