#![allow(clippy::type_complexity, clippy::too_many_arguments)]

use std::time::Duration;

use bevy::{
    app::{AppExit, PluginGroupBuilder, ScheduleRunnerPlugin},
    core_pipeline::CorePipelinePlugin,
    gltf::GltfPlugin,
    pbr::PbrPlugin,
    prelude::*,
    render::{
        camera::{CameraPlugin, CameraProjectionPlugin},
        extract_resource::ExtractResource,
        pipelined_rendering::PipelinedRenderingPlugin,
        RenderPlugin,
    },
};
use bevy_mod_picking::DefaultPickingPlugins;

use bevy_web_asset::WebAssetPlugin;
use bvh::{BvhAnimCompleteEvent, BvhPlugin};
use input::StudioInputSystem;
use scene::TestBvh;
use screen_shot::{ScreenCapture, ScreenShotPlugin};
use space_editor::prelude::{EditorEvent, EditorPrefabPath};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;
use wgpu::Extent3d;

mod bvh;
mod camera_controller;
mod debug;
mod design_system;
mod input;
mod inspector;
mod interaction;
mod math;
mod persistance;
mod scene;
pub mod screen_shot;
mod skybox;
mod ui;
pub mod viewport;
mod world_grid;

pub use crate::{
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
#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug)]
pub enum StudioMode {
    Editor,
    Headless,
    Viewer,
}

pub(crate) fn is_editor(r_studio_mode: Res<StudioMode>) -> bool {
    matches!(*r_studio_mode, StudioMode::Editor)
}

/// The collection of plugins that should be loaded for both viewer-mode and
/// editor-mode instances of the app.
pub struct CommonPlugins;
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
            .add(persistance::PersistancePlugin)
            .add(BvhPlugin)
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

    app.add_systems(
        Startup,
        move |mut cmd: Commands, r_assets: Res<AssetServer>| {
            cmd.insert_resource(scene::TestBvh(r_assets.load(bvh_path.clone())));
        },
    );

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

pub enum HeadlessMode {
    Bvh(String, String),
    Scene(String, String),
}

pub fn start_headless(
    mode: HeadlessMode,
    fps: u32,
    outout_dir: &str,
    frames_to_capture: u32,
    image_res: Extent3d,
) -> App {
    let mut app = App::new();

    app.insert_resource(StudioMode::Headless);
    app.insert_resource(HeadlessSettings {
        frames_to_capture,
        output_dims: image_res,
        output_dir: outout_dir.to_string(),
    });

    app.add_plugins((
        ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / fps as f64)),
        DefaultPlugins.set(WindowPlugin {
            primary_window: None,
            exit_condition: bevy::window::ExitCondition::DontExit,
            close_when_requested: true,
        }),
        ScreenShotPlugin,
        CommonPlugins,
    ));

    match mode {
        HeadlessMode::Bvh(bvh_dir, skybox) => {
            app.add_systems(
                Startup,
                move |mut cmd: Commands,
                      asset_server: Res<AssetServer>,
                      mut ew: EventWriter<LoadSceneEvent>| {
                    cmd.insert_resource(SkyboxId(skybox.clone()));
                    cmd.insert_resource(TestBvh(asset_server.load(bvh_dir.clone())));

                    ew.send(LoadSceneEvent("Mannequin.gltf".to_string()));
                },
            );
        }
        HeadlessMode::Scene(scene, skybox) => {
            app.add_systems(
                Startup,
                move |mut cmd: Commands, mut ew: EventWriter<EditorEvent>| {
                    cmd.insert_resource(SkyboxId(skybox.clone()));
                    ew.send(EditorEvent::Load(EditorPrefabPath::File(scene.clone())))
                },
            );
        }
    }

    // app.add_event::<LoadSceneEvent>();
    // if let Some(scene) = scene {
    //     app.world.send_event(EditorEvent::Load(EditorPrefabPath::File(scene)));
    // }

    // app.add_systems(
    //     Startup,
    //     move |mut cmd: Commands, mut ew: EventWriter<LoadSceneEvent>| {
    //         cmd.insert_resource(SkyboxId(skybox_id.clone()));
    //         ew.send(LoadSceneEvent(object_id.clone()));i
    //     },
    // );

    app.add_systems(PostUpdate, check_for_capture_complete);

    // #[cfg(feature = "wasm")]
    // wasm::initialize();

    app
}

pub fn is_headless(studio_state: Res<StudioMode>) -> bool {
    *studio_state == StudioMode::Headless
}

pub fn is_not_headless(studio_state: Res<StudioMode>) -> bool {
    *studio_state != StudioMode::Headless
}

#[derive(Resource)]
pub struct HeadlessSettings {
    pub output_dims: Extent3d,
    pub output_dir: String,
    pub frames_to_capture: u32,
}

impl Default for HeadlessSettings {
    fn default() -> Self {
        HeadlessSettings {
            output_dims: Extent3d {
                width: 1280,
                height: 720,
                depth_or_array_layers: 1,
            },
            output_dir: "./screen_shots/".to_string(),
            frames_to_capture: 1,
        }
    }
}

fn check_for_capture_complete(
    main_camera: Query<&ScreenCapture, With<MainCamera>>,
    mut close_app: EventWriter<AppExit>,
    mut bvh_event_reader: EventReader<BvhAnimCompleteEvent>,
) {
    let Ok(screen_capture) = main_camera.get_single() else {
        return;
    };

    if screen_capture.is_complete() || !bvh_event_reader.is_empty() {
        close_app.send(AppExit);
    }
}
