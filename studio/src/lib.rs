#![allow(clippy::type_complexity, clippy::too_many_arguments)]

use bevy::{app::PluginGroupBuilder, prelude::*};
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_web_asset::WebAssetPlugin;
use scene::ImportSceneEvent;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

mod anim;
mod bvh;
mod camera_controller;
mod change_tracking;
mod debug;
mod design_system;
pub mod headless;
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
#[repr(u8)]
#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug, Deserialize_repr, Serialize_repr)]
pub enum StudioMode {
    Editor = 0,
    Headless = 1,
    Viewer = 2,
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

#[cfg(feature = "wasm")]
#[wasm_bindgen(typescript_custom_section)]
const WASM_ENTRY_POINTS: &str = r#"
export type SkyboxId
    = "gum_trees_4k"
    | "kloofenfal_28d_misty_4k"
    | "meadow_4k"
    | "promenade_de_vidy_4k"
    | "scythian_tombs_4k"
    | "test_scene"
    ;

export interface InitOptions {
    mode: StudioMode;
    /** Required for all entry-points except `initFromStorytellerScene()` */
    skyboxId?: SkyboxId;
    canvasSelector: string;
    canvasAlt: string;
    viewportSize: ViewportSize;
}

export function initFromLibrary(objectId: string, options: InitOptions): void;
export function initFromBvh(animPath: string, options: InitOptions): void;
export function initFromMixamo(animPath: string, options: InitOptions): void;
export function initFromSceneImport(scenePath: string, options: InitOptions): void;
export function initFromStorytellerScene(scenePath: string, options: InitOptions): void;
"#;

#[derive(Serialize, Deserialize)]
struct WasmInitOptions {
    #[serde(rename = "skyboxId")]
    skybox_id: Option<String>,
    mode: StudioMode,
    #[serde(rename = "canvasSelector")]
    canvas_selector: String,
    #[serde(rename = "canvasAlt")]
    canvas_alt: String,
    #[serde(rename = "viewportSize")]
    viewport_size: ViewportSize,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen(skip_typescript, js_name = initFromLibrary)]
pub fn init_from_library(object_id: String, options: JsValue) -> Result<(), JsValue> {
    let options = serde_wasm_bindgen::from_value::<WasmInitOptions>(options)?;
    let skybox_id = options
        .skybox_id
        .expect("`skyboxId` is required for `initFromLibrary`");

    start_internal(
        InitParams::FromAsset {
            asset: InitAsset::LibraryObject(object_id),
            skybox_id,
        },
        options.mode,
        Some(Window {
            resolution: options.viewport_size.into(),
            title: options.canvas_alt,
            canvas: Some(options.canvas_selector),
            ..default()
        }),
    )
    .run();

    Ok(())
}

#[cfg(feature = "wasm")]
#[wasm_bindgen(skip_typescript, js_name = initFromBvh)]
pub fn init_from_bvh(anim_path: String, options: JsValue) -> Result<(), JsValue> {
    let options = serde_wasm_bindgen::from_value::<WasmInitOptions>(options)?;
    let skybox_id = options
        .skybox_id
        .expect("`skyboxId` is required for `initFromBvh`");

    start_internal(
        InitParams::FromAsset {
            asset: InitAsset::BvhAnim(anim_path),
            skybox_id,
        },
        options.mode,
        Some(Window {
            resolution: options.viewport_size.into(),
            title: options.canvas_alt,
            canvas: Some(options.canvas_selector),
            ..default()
        }),
    )
    .run();

    Ok(())
}

#[cfg(feature = "wasm")]
#[wasm_bindgen(skip_typescript, js_name = initFromMixamo)]
pub fn init_from_mixamo(anim_path: String, options: JsValue) -> Result<(), JsValue> {
    let options = serde_wasm_bindgen::from_value::<WasmInitOptions>(options)?;
    let skybox_id = options
        .skybox_id
        .expect("`skyboxId` is required for `initFromMixamo`");

    start_internal(
        InitParams::FromAsset {
            asset: InitAsset::MixamoAnim(anim_path),
            skybox_id,
        },
        options.mode,
        Some(Window {
            resolution: options.viewport_size.into(),
            title: options.canvas_alt,
            canvas: Some(options.canvas_selector),
            ..default()
        }),
    )
    .run();

    Ok(())
}

#[cfg(feature = "wasm")]
#[wasm_bindgen(skip_typescript, js_name = initFromSceneImport)]
pub fn init_from_scene_import(scene_path: String, options: JsValue) -> Result<(), JsValue> {
    let options = serde_wasm_bindgen::from_value::<WasmInitOptions>(options)?;
    let skybox_id = options
        .skybox_id
        .expect("`skyboxId` is required for `initFromSceneImport`");

    start_internal(
        InitParams::FromAsset {
            asset: InitAsset::ImportedScene(scene_path),
            skybox_id,
        },
        options.mode,
        Some(Window {
            resolution: options.viewport_size.into(),
            title: options.canvas_alt,
            canvas: Some(options.canvas_selector),
            ..default()
        }),
    )
    .run();

    Ok(())
}

#[cfg(feature = "wasm")]
#[wasm_bindgen(skip_typescript, js_name = initFromStorytellerScene)]
pub fn init_from_storyteller_scene(scene_path: String, options: JsValue) -> Result<(), JsValue> {
    let options = serde_wasm_bindgen::from_value::<WasmInitOptions>(options)?;

    start_internal(
        InitParams::FromScene(scene_path),
        options.mode,
        Some(Window {
            resolution: options.viewport_size.into(),
            title: options.canvas_alt,
            canvas: Some(options.canvas_selector),
            ..default()
        }),
    )
    .run();

    Ok(())
}

// #[cfg_attr(feature = "wasm", wasm_bindgen(js_name = startViewer))]
// /// Start the application as a read-only viewer for a single library object.
// ///
// /// @param object_id ID of the library object to load.
// /// @param skybox_id ID of the Skybox to use
// /// @param canvas_selector A CSS selector for the canvas that will host the Bevy
// ///   application. This should be unique (i.e. `querySelectorAll` should not
// ///   return more than one element), so using an ID selector is a good idea.
// /// @param canvas_alt `alt` text for the HTML canvas element.
// /// @param viewport_size The size of the canvas in CSS `px` units.
// pub fn start_viewer(
//     object_id: &str,
//     skybox_id: &str,
//     canvas_selector: &str,
//     canvas_alt: &str,
//     viewport_size: ViewportSize,
// ) {
//     start_internal(
//         InitParams::FromLibrary {
//             object_id: object_id.into(),
//             skybox_id: skybox_id.into(),
//         },
//         canvas_selector,
//         canvas_alt,
//         viewport_size,
//         StudioMode::Viewer,
//     )
//     .run();
// }

// #[cfg_attr(feature = "wasm", wasm_bindgen(js_name = startEditor))]
// pub fn start_editor(
//     scene_id: &str,
//     canvas_selector: &str,
//     canvas_alt: &str,
//     viewport_size: ViewportSize,
// ) {
//     start_internal(
//         InitParams::FromScene(scene_id.into()),
//         canvas_selector,
//         canvas_alt,
//         viewport_size,
//         StudioMode::Editor,
//     )
//     .run();
// }

// #[cfg_attr(feature = "wasm", wasm_bindgen(js_name = startBvhViewer))]
// pub fn start_bvh_viewer(
//     bvh_path: &str,
//     skybox_id: &str,
//     canvas_selector: &str,
//     canvas_alt: &str,
//     viewport_size: ViewportSize,
// ) {
//     let mut app = start_internal(
//         InitParams::FromLibrary {
//             object_id: "Mannequin.gltf".into(),
//             skybox_id: skybox_id.into(),
//         },
//         canvas_selector,
//         canvas_alt,
//         viewport_size,
//         StudioMode::Viewer,
//     );

//     let bvh_path = bvh_path.to_string();

//     app.add_plugins(BvhPlugin);
//     app.add_systems(
//         Startup,
//         move |mut cmd: Commands, r_assets: Res<AssetServer>| {
//             cmd.insert_resource(bvh::TestBvh(r_assets.load(bvh_path.clone())));
//         },
//     );

//     app.run();
// }

// #[cfg_attr(feature = "wasm", wasm_bindgen(js_name = startMixamoViewer))]
// pub fn start_mixamo_viewer(
//     mixamo_path: &str,
//     skybox_id: &str,
//     canvas_selector: &str,
//     canvas_alt: &str,
//     viewport_size: ViewportSize,
// ) {
//     let mut app = start_internal(
//         InitParams::FromLibrary {
//             object_id: "Mannequin.gltf".into(),
//             skybox_id: skybox_id.into(),
//         },
//         canvas_selector,
//         canvas_alt,
//         viewport_size,
//         StudioMode::Viewer,
//     );

//     let mixamo_path = mixamo_path.to_string();

//     app.add_plugins(MixamoPlugin);
//     app.add_systems(Startup, move |mut ew: EventWriter<MixamoImportEvent>| {
//         ew.send(MixamoImportEvent(mixamo_path.clone()));
//     });

//     app.run();
// }

// #[cfg_attr(feature = "wasm", wasm_bindgen)]
// pub fn start(
//     object_id: &str,
//     skybox_id: &str,
//     canvas_selector: &str,
//     canvas_alt: &str,
//     viewport_size: ViewportSize,
//     studio_mode: StudioMode,
// ) {
//     start_internal(
//         InitParams::FromLibrary {
//             object_id: object_id.into(),
//             skybox_id: skybox_id.into(),
//         },
//         canvas_selector,
//         canvas_alt,
//         viewport_size,
//         studio_mode,
//     )
//     .run();
// }

#[derive(Debug)]
pub enum InitParams {
    /// Initialize the application by importing an asset and processing it to
    /// work with Storyteller Studio's feature-set.
    FromAsset {
        asset: InitAsset,
        /// A skybox name that corresponds to a matching set of assets under
        /// `studio/assets/skyboxes`, _not_ including file extensions or
        /// "diffuse"/"specular" qualifiers (e.g., "gum_trees_4k")
        skybox_id: String,
    },
    /// Initialize the application by loading a previously saved Storyteller
    /// Studio scene.
    FromScene(String),
}

#[derive(Debug)]
pub enum InitAsset {
    /// Initialize the application from an asset in the "bundled" asset library.
    /// Takes a filename, including the extension, that can be found under
    /// `studio/assets/gltf` (e.g., "base-human-female.gltf").
    LibraryObject(String),
    /// Initialize the application by loading an arbitrary glTF scene from any
    /// source (e.g., the Storyteller.ai backend)
    ImportedScene(String),
    /// Initialize the application by retargeting a BVH animation (currently
    /// only supports the MocapNET skeleton) onto the bundled `Mannequin.gltf`
    /// model.
    BvhAnim(String),
    /// Initialize the application by retargeting a Mixamo animation onto the
    /// bundled `Mannequin.gltf` model.
    MixamoAnim(String),
}

pub fn start_internal(init: InitParams, mode: StudioMode, window_config: Option<Window>) -> App {
    #[cfg(feature = "wasm")]
    console_error_panic_hook::set_once();

    let mut app = App::new();

    app.insert_resource(mode);

    app.add_plugins((
        #[cfg(feature = "wasm")]
        viewport::wasm::ViewportPlugin,
        WebAssetPlugin,
        RemoteAssetPlugin,
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(window_config.unwrap_or_default()),
            // primary_window: Some(Window {
            //     resolution: viewport_size.into(),
            //     title: canvas_alt.into(),
            //     canvas: Some(canvas_selector.into()),
            //     ..default()
            // }),
            ..default()
        }),
        CommonPlugins,
        #[cfg(all(feature = "wasm", feature = "wasm_screenshot"))]
        wasm::ScreenshotPlugin,
    ));

    if matches!(mode, StudioMode::Editor) {
        app.add_plugins((DefaultPickingPlugins, EditorPlugins));

        // Only for registering components for undo
        use space_editor::prelude::space_undo::AppAutoUndo;
        app.auto_reflected_undo::<Transform>();
    }

    match init {
        InitParams::FromAsset { asset, skybox_id } => {
            app.add_systems(Startup, move |mut cmd: Commands| {
                cmd.insert_resource(SkyboxId(skybox_id.clone()));
            });

            match asset {
                InitAsset::LibraryObject(id) => {
                    app.add_systems(
                        Startup,
                        move |mut ew: EventWriter<LoadLibraryObjectEvent>| {
                            ew.send(LoadLibraryObjectEvent(id.clone()));
                        },
                    );
                }
                anim @ (InitAsset::BvhAnim(_) | InitAsset::MixamoAnim(_)) => {
                    app.add_systems(
                        Startup,
                        move |mut ew: EventWriter<LoadLibraryObjectEvent>| {
                            ew.send(LoadLibraryObjectEvent("Mannequin.gltf".into()));
                        },
                    );

                    match anim {
                        InitAsset::BvhAnim(anim) => {
                            app.add_plugins(BvhPlugin);
                            app.add_systems(
                                Startup,
                                move |mut cmd: Commands, r_assets: Res<AssetServer>| {
                                    cmd.insert_resource(bvh::TestBvh(r_assets.load(anim.clone())));
                                },
                            );
                        }
                        InitAsset::MixamoAnim(anim) => {
                            app.add_plugins(MixamoPlugin);
                            app.add_systems(
                                Startup,
                                move |mut ew: EventWriter<MixamoImportEvent>| {
                                    ew.send(MixamoImportEvent(anim.clone()));
                                },
                            );
                        }
                        _ => unreachable!(),
                    }
                }
                InitAsset::ImportedScene(scene) => {
                    app.add_systems(Startup, move |mut ew: EventWriter<ImportSceneEvent>| {
                        ew.send(ImportSceneEvent(scene.clone()));
                    });
                }
            }
        }
        InitParams::FromScene(scene) => {
            app.add_systems(Startup, move |mut ew: EventWriter<LoadSceneEvent>| {
                ew.send(LoadSceneEvent(scene.clone()));
            });
        }
    }

    #[cfg(feature = "wasm")]
    wasm::initialize();

    app
}
