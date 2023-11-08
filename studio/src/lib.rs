#![allow(clippy::type_complexity, clippy::too_many_arguments)]

use bevy::prelude::*;
use bevy_mod_picking::DefaultPickingPlugins;
use camera_controller::CameraControllerPlugin;
use debug::DebugPlugin;
use input::StudioInputPlugin;
use interaction::InteractionPlugin;
use math::InterpolationPlugin;
use scene::ScenePlugin;
use skybox::SkyboxPlugin;
use ui::UiLayerPlugin;
use viewport::ViewportSize;
use wasm_bindgen::prelude::*;

mod camera_controller;
mod debug;
mod design_system;
mod input;
mod interaction;
mod math;
mod scene;
mod skybox;
mod ui;
mod viewport;

#[cfg(feature = "wasm")]
mod wasm;

#[derive(Component)]
pub(crate) struct MainCamera;

#[wasm_bindgen]
pub fn init(viewport_size: ViewportSize) {
    #[cfg(feature = "wasm")]
    console_error_panic_hook::set_once();

    let mut app = App::new();

    #[cfg(feature = "wasm")]
    app.add_plugins(viewport::wasm::ViewportPlugin);

    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: viewport_size.into(),
                title: "Storyteller Studio".into(),
                canvas: Some("#studio_canvas".into()),
                ..default()
            }),
            ..default()
        }),
        DefaultPickingPlugins,
        DebugPlugin,
        InteractionPlugin,
        StudioInputPlugin,
        InterpolationPlugin,
        CameraControllerPlugin,
        ScenePlugin,
        SkyboxPlugin,
        UiLayerPlugin,
    ))
    .run();
}
