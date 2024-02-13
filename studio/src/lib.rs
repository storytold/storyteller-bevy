#![allow(clippy::type_complexity, clippy::too_many_arguments)]

use bevy::{app::PluginGroupBuilder, prelude::*};
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_web_asset::WebAssetPlugin;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

mod anim;
mod bvh;
mod camera_controller;
mod debug;
mod design_system;
mod hierarchy;
mod input;
mod inspector;
mod interaction;
pub mod math;
mod mixamo;
mod persistance;
mod scene;
mod skybox;
mod ui;
pub mod viewport;
mod world_grid;

pub use crate::{
    anim::AnimPlugin,
    bvh::BvhPlugin,
    camera_controller::CameraControllerPlugin,
    debug::DebugPlugin,
    input::StudioInputPlugin,
    inspector::InspectorPlugin,
    interaction::InteractionPlugin,
    math::InterpolationPlugin,
    mixamo::{MixamoImportEvent, MixamoPlugin},
    persistance::PersistancePlugin,
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
#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug)]
pub enum StudioMode {
    Editor,
    Viewer,
}

pub(crate) fn is_editor(r_studio_mode: Res<StudioMode>) -> bool {
    matches!(*r_studio_mode, StudioMode::Editor)
}

/// The collection of plugins that should be loaded for both viewer-mode and
/// editor-mode instances of the app.
struct CommonPlugins;
impl PluginGroup for CommonPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(CameraControllerPlugin)
            .add(DebugPlugin)
            .add(InspectorPlugin)
            .add(InterpolationPlugin)
            .add(ScenePlugin)
            .add(SkyboxPlugin)
            .add(StudioInputPlugin)
            .add(UiLayerPlugin)
            .add(WorldGridPlugin)
            .add(PersistancePlugin)
            .add(AnimPlugin)
    }
}

/// The collection of plugins that should be loaded only for editor-mode
/// instances of the app.
struct EditorPlugins;
impl PluginGroup for EditorPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(InteractionPlugin)
    }
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
    start_internal(
        object_id,
        skybox_id,
        canvas_selector,
        canvas_alt,
        viewport_size,
        StudioMode::Viewer,
    )
    .run();
}

#[cfg_attr(feature = "wasm", wasm_bindgen(js_name = startEditor))]
pub fn start_editor(
    object_id: &str,
    skybox_id: &str,
    canvas_selector: &str,
    canvas_alt: &str,
    viewport_size: ViewportSize,
) {
    start_internal(
        object_id,
        skybox_id,
        canvas_selector,
        canvas_alt,
        viewport_size,
        StudioMode::Editor,
    )
    .run();
}

#[cfg_attr(feature = "wasm", wasm_bindgen(js_name = startBvhViewer))]
pub fn start_bvh_viewer(
    bvh_path: &str,
    skybox_id: &str,
    canvas_selector: &str,
    canvas_alt: &str,
    viewport_size: ViewportSize,
) {
    let mut app = start_internal(
        "Mannequin.gltf",
        skybox_id,
        canvas_selector,
        canvas_alt,
        viewport_size,
        StudioMode::Viewer,
    );

    let bvh_path = bvh_path.to_string();

    app.add_plugins(BvhPlugin);
    app.add_systems(
        Startup,
        move |mut cmd: Commands, r_assets: Res<AssetServer>| {
            cmd.insert_resource(bvh::TestBvh(r_assets.load(bvh_path.clone())));
        },
    );

    app.run();
}

#[cfg_attr(feature = "wasm", wasm_bindgen(js_name = startMixamoViewer))]
pub fn start_mixamo_viewer(
    mixamo_path: &str,
    skybox_id: &str,
    canvas_selector: &str,
    canvas_alt: &str,
    viewport_size: ViewportSize,
) {
    let mut app = start_internal(
        "Mannequin.gltf",
        skybox_id,
        canvas_selector,
        canvas_alt,
        viewport_size,
        StudioMode::Viewer,
    );

    let mixamo_path = mixamo_path.to_string();

    app.add_plugins(MixamoPlugin);
    app.add_systems(Startup, move |mut ew: EventWriter<MixamoImportEvent>| {
        ew.send(MixamoImportEvent(mixamo_path.clone()));
    });

    app.run();
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn start(
    object_id: &str,
    skybox_id: &str,
    canvas_selector: &str,
    canvas_alt: &str,
    viewport_size: ViewportSize,
    studio_mode: StudioMode,
) {
    start_internal(
        object_id,
        skybox_id,
        canvas_selector,
        canvas_alt,
        viewport_size,
        studio_mode,
    )
    .run();
}

// TODO: These `start_` functiions should be refactored to better support the
//       needs of the "reaodnly viewer" vs "full studio editor" use cases. See
//       notes in `studio-web/src/lib/viewer.element.ts` for more info.

pub fn start_internal(
    object_id: &str,
    skybox_id: &str,
    canvas_selector: &str,
    canvas_alt: &str,
    viewport_size: ViewportSize,
    studio_mode: StudioMode,
) -> App {
    #[cfg(feature = "wasm")]
    console_error_panic_hook::set_once();

    let object_id = object_id.to_string();
    let skybox_id = skybox_id.to_string();

    let mut app = App::new();

    app.insert_resource(studio_mode);

    app.add_plugins((
        #[cfg(feature = "wasm")]
        viewport::wasm::ViewportPlugin,
        WebAssetPlugin,
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: viewport_size.into(),
                title: canvas_alt.into(),
                canvas: Some(canvas_selector.into()),
                ..default()
            }),
            ..default()
        }),
        CommonPlugins,
    ));

    if matches!(studio_mode, StudioMode::Editor) {
        app.add_plugins((DefaultPickingPlugins, EditorPlugins));

        // Only for registering components for undo
        use space_editor::prelude::space_undo::AppAutoUndo;
        app.auto_reflected_undo::<Transform>();
    }

    app.add_systems(
        Startup,
        move |mut cmd: Commands, mut ew: EventWriter<LoadSceneEvent>| {
            cmd.insert_resource(SkyboxId(skybox_id.clone()));
            ew.send(LoadSceneEvent(object_id.clone()));
        },
    );

    #[cfg(feature = "wasm")]
    wasm::initialize();

    app
}
