#![allow(clippy::type_complexity, clippy::too_many_arguments)]

use std::time::Duration;

use bevy::{
    app::{AppExit, PluginGroupBuilder, ScheduleRunnerPlugin},
    prelude::*,
};
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_web_asset::WebAssetPlugin;
use bvh::{BvhAnimCompleteEvent, TestBvh};
use screenshot::{ScreenCapture, ScreenShotPlugin};
use space_editor::prelude::{EditorEvent, EditorPrefabPath};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;
use wgpu::Extent3d;

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
mod path;
mod persistance;
mod remote;
mod scene;
pub mod screenshot;
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
    remote::RemoteAssetPlugin,
    scene::{LoadLibraryObjectEvent, LoadSceneEvent, ScenePlugin},
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
        InitSeed::FromLibrary {
            object_id: object_id.into(),
            skybox_id: skybox_id.into(),
        },
        canvas_selector,
        canvas_alt,
        viewport_size,
        StudioMode::Viewer,
    )
    .run();
}

#[cfg_attr(feature = "wasm", wasm_bindgen(js_name = startEditor))]
pub fn start_editor(
    scene_id: &str,
    canvas_selector: &str,
    canvas_alt: &str,
    viewport_size: ViewportSize,
) {
    start_internal(
        InitSeed::FromScene(scene_id.into()),
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
        InitSeed::FromLibrary {
            object_id: "Mannequin.gltf".into(),
            skybox_id: skybox_id.into(),
        },
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
        InitSeed::FromLibrary {
            object_id: "Mannequin.gltf".into(),
            skybox_id: skybox_id.into(),
        },
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
        InitSeed::FromLibrary {
            object_id: object_id.into(),
            skybox_id: skybox_id.into(),
        },
        canvas_selector,
        canvas_alt,
        viewport_size,
        studio_mode,
    )
    .run();
}

pub enum InitSeed {
    /// Initialize the application from assets in the "bundled" asset library.
    FromLibrary {
        /// A filename, including the extension, that can be found under
        /// `studio/assets/gltf` (e.g., "Mannequin.gltf")
        object_id: String,
        /// A skybox name that corresponds to a matching set of assets under
        /// `studio/assets/skyboxes`, _not_ including file extensions or
        /// "diffuse"/"specular" qualifiers (e.g., "gum_trees_4k")
        skybox_id: String,
    },
    /// Initialize the application by loading a previously saved `*.scn.ron`
    /// scene file.
    FromScene(String),
}

pub fn start_internal(
    seed: InitSeed,
    canvas_selector: &str,
    canvas_alt: &str,
    viewport_size: ViewportSize,
    studio_mode: StudioMode,
) -> App {
    #[cfg(feature = "wasm")]
    console_error_panic_hook::set_once();

    let mut app = App::new();

    app.insert_resource(HeadlessMode::None);
    app.insert_resource(studio_mode);

    app.add_plugins((
        #[cfg(feature = "wasm")]
        viewport::wasm::ViewportPlugin,
        WebAssetPlugin,
        RemoteAssetPlugin,
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

    match seed {
        InitSeed::FromLibrary {
            object_id,
            skybox_id,
        } => {
            app.add_systems(
                Startup,
                move |mut cmd: Commands, mut ew: EventWriter<LoadLibraryObjectEvent>| {
                    cmd.insert_resource(SkyboxId(skybox_id.clone()));
                    ew.send(LoadLibraryObjectEvent(object_id.clone()));
                },
            );
        }
        InitSeed::FromScene(scene) => {
            app.add_systems(Startup, move |mut ew: EventWriter<LoadSceneEvent>| {
                ew.send(LoadSceneEvent(scene.clone()));
            });
        }
    }

    #[cfg(feature = "wasm")]
    wasm::initialize();

    app
}

#[derive(Debug, Resource, Clone)]
pub enum HeadlessMode {
    Bvh(String, String),
    Scene(String, String),
    Mixamo(String, String),
    None,
}

pub fn start_headless(
    mode: HeadlessMode,
    fps: u32,
    outout_dir: &str,
    frames_to_capture: u32,
    image_res: Extent3d,
) -> App {
    let mut app = App::new();

    app.insert_resource(mode.clone());
    app.insert_resource(StudioMode::Headless);
    app.insert_resource(HeadlessSettings {
        max_frames: frames_to_capture,
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
        CommonPlugins.build().disable::<WorldGridPlugin>(),
    ));

    match mode {
        HeadlessMode::Bvh(bvh_dir, skybox) => {
            app.add_plugins(BvhPlugin);
            app.add_systems(
                Startup,
                move |mut cmd: Commands,
                      asset_server: Res<AssetServer>,
                      mut ew: EventWriter<LoadLibraryObjectEvent>| {
                    cmd.insert_resource(SkyboxId(skybox.clone()));
                    cmd.insert_resource(TestBvh(asset_server.load(bvh_dir.clone())));
                    ew.send(LoadLibraryObjectEvent("Mannequin.gltf".to_string()));
                },
            );

            app.add_systems(PostUpdate, check_for_bvh_capture_complete);
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
        HeadlessMode::Mixamo(mixamo_path, skybox) => {
            app.add_plugins(MixamoPlugin);
            app.add_systems(
                Startup,
                move |mut commands: Commands,
                      mut ew: EventWriter<MixamoImportEvent>,
                      mut load_scene_event: EventWriter<LoadSceneEvent>| {
                    ew.send(MixamoImportEvent(mixamo_path.clone()));
                    load_scene_event.send(LoadSceneEvent("Mannequin.gltf".to_string()));
                    commands.insert_resource(SkyboxId(skybox.clone()));
                },
            );
            app.add_systems(PostUpdate, check_for_mixamo_capture_complete);

            app.run();
        }
        HeadlessMode::None => {}
    }

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
    pub max_frames: u32,
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
            max_frames: u32::MAX,
        }
    }
}

fn check_for_bvh_capture_complete(
    mut close_app: EventWriter<AppExit>,
    mut main_camera: Query<&mut ScreenCapture, With<MainCamera>>,
    bvh_event_reader: EventReader<BvhAnimCompleteEvent>,
) {
    let Ok(mut screen_capture) = main_camera.get_single_mut() else {
        return;
    };

    println!("Frames captured {}", screen_capture.frames_captured());
    if !screen_capture.should_capture() || !bvh_event_reader.is_empty() {
        screen_capture.stop_capturing();
        close_app.send(AppExit);
    }
}

fn check_for_mixamo_capture_complete(
    mut close_app: EventWriter<AppExit>,
    mut main_camera: Query<&mut ScreenCapture, With<MainCamera>>,
    mixamo_anims: Query<&AnimationPlayer>,
) {
    let Ok(mut capture) = main_camera.get_single_mut() else {
        return;
    };

    for anim in mixamo_anims.iter() {
        println!(
            "Time elapsed {} | Frames Captured : {}",
            anim.elapsed(),
            capture.frames_captured()
        );
        if anim.completions() != 0 {
            capture.stop_capturing();
            close_app.send(AppExit);
        }
    }
}
