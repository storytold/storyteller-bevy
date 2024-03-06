use bevy::prelude::*;

use self::fps::FpsCounterPlugin;

mod fps;

#[cfg(feature = "wasm")]
mod blocking;

#[cfg(feature = "wasm")]
pub mod inspector;

pub struct UiLayerPlugin;

impl Plugin for UiLayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FpsCounterPlugin);

        #[cfg(feature = "wasm")]
        {
            app.add_plugins(inspector::InspectorPlugin);
            app.add_plugins(blocking::UiBlockingPlugin);
        }
    }
}
