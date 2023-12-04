#![allow(clippy::type_complexity, clippy::too_many_arguments)]

use bevy::prelude::*;
use bevy_mod_picking::DefaultPickingPlugins;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

mod camera_controller;
mod debug;
mod design_system;
mod input;
mod inspector;
mod interaction;
mod math;
mod scene;
mod skybox;
mod ui;
mod viewport;
mod world_grid;

use crate::{
    camera_controller::CameraControllerPlugin,
    debug::DebugPlugin,
    input::StudioInputPlugin,
    inspector::InspectorPlugin,
    interaction::InteractionPlugin,
    math::InterpolationPlugin,
    scene::{LoadSceneEvent, ScenePlugin},
    skybox::{SkyboxId, SkyboxPlugin},
    ui::UiLayerPlugin,
    viewport::ViewportSize,
    world_grid::WorldGridPlugin,
};

#[cfg(feature = "wasm")]
mod wasm;

#[derive(Component)]
pub(crate) struct MainCamera;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn init(viewport_size: ViewportSize) {
    #[cfg(feature = "wasm")]
    console_error_panic_hook::set_once();

    let mut app = App::new();

    #[cfg(feature = "wasm")]
    app.add_plugins(viewport::wasm::ViewportPlugin);

    app.add_plugins((
        DefaultPlugins
            // TODO: To generate asset .meta files (to prevent 404 errors on the
            //       web), it's currently necessary to uncomment the following
            //       block and run the app with this command:
            //     > cargo run -p studio --features bevy/asset_processor,bevy/tonemapping_luts
            // .set(AssetPlugin {
            //     mode: AssetMode::Processed,
            //     ..default()
            // })
            .set(WindowPlugin {
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
        InspectorPlugin,
        StudioInputPlugin,
        InterpolationPlugin,
        CameraControllerPlugin,
        ScenePlugin,
        SkyboxPlugin,
        UiLayerPlugin,
        WorldGridPlugin,
    ));

    #[cfg(feature = "wasm")]
    wasm::initialize();

    app.run();
}

#[cfg_attr(feature = "wasm", wasm_bindgen(js_name = startViewer))]
/// Start the application as a read-only viewer for a single library object.
///
/// @param object_id ID of the library object to load.
/// @param skybox_id ID of the Skybox to use
/// @param canvas_selector A CSS selector for the canvas that will host the Bevy
///   application. This should be unique (i.e. `querySelectorAll` should not
///   return more than one element), so using an ID selector is a good idea.
/// @param canvas_alt `alt` text for the HTML canvas element.
/// @param viewport_size The size of the canvas in CSS `px` units.
pub fn start_viewer(
    object_id: &str,
    skybox_id: &str,
    canvas_selector: &str,
    canvas_alt: &str,
    viewport_size: ViewportSize,
) {
    #[cfg(feature = "wasm")]
    console_error_panic_hook::set_once();

    let object_id = object_id.to_string();
    let skybox_id = skybox_id.to_string();

    let mut app = App::new();
    app.add_plugins((
        #[cfg(feature = "wasm")]
        viewport::wasm::ViewportPlugin,
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: viewport_size.into(),
                title: canvas_alt.into(),
                canvas: Some(canvas_selector.into()),
                ..default()
            }),
            ..default()
        }),
        DefaultPickingPlugins,
        DebugPlugin,
        InteractionPlugin,
        InspectorPlugin,
        StudioInputPlugin,
        InterpolationPlugin,
        CameraControllerPlugin,
        ScenePlugin,
        SkyboxPlugin,
        UiLayerPlugin,
        WorldGridPlugin,
    ))
    .add_systems(
        Startup,
        move |mut cmd: Commands, mut ew: EventWriter<LoadSceneEvent>| {
            cmd.insert_resource(SkyboxId(skybox_id.clone()));
            ew.send(LoadSceneEvent(object_id.clone()));
        },
    );

    #[cfg(feature = "wasm")]
    wasm::initialize();

    app.run();
}
