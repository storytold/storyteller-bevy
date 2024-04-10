#![allow(clippy::type_complexity, clippy::too_many_arguments)]

use bevy::{app::PluginGroupBuilder, asset::AssetMetaCheck, prelude::*};
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_web_asset::WebAssetPlugin;
use scene::ImportSceneEvent;

#[cfg(feature = "wasm")]
use serde_repr::{Deserialize_repr, Serialize_repr};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

mod anim;
mod bvh;
mod camera_controller;
mod change_tracking;
mod debug;
mod design_system;
#[cfg(feature = "headless")]
pub mod headless;
mod hierarchy;
mod input;
mod inspector;
mod interaction;
pub mod material;
pub mod math;
mod mixamo;
mod path;
mod persistance;
mod remote;
pub mod scene;
#[cfg(feature = "headless")]
pub mod screenshot;
mod skybox;
mod ui;
pub mod viewport;
mod world_grid;

use crate::material::MaterialPlugin;
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

#[cfg_attr(
    feature = "wasm",
    wasm_bindgen,
    repr(u8),
    derive(Deserialize_repr, Serialize_repr)
)]
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
            .add(MaterialPlugin)
            // .add(SkyboxPlugin)
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

pub fn start(init: InitParams, mode: StudioMode, window_config: Option<Window>) -> App {
    #[cfg(feature = "wasm")]
    console_error_panic_hook::set_once();

    let mut app = App::new();

    app.insert_resource(mode);
    app.insert_resource(bevy::winit::WinitSettings {
        focused_mode: bevy::winit::UpdateMode::Continuous,
        unfocused_mode: bevy::winit::UpdateMode::Continuous,
    });
    app.insert_resource(AssetMetaCheck::Never);

    app.add_plugins((
        #[cfg(feature = "wasm")]
        viewport::wasm::ViewportPlugin,
        WebAssetPlugin,
        RemoteAssetPlugin,
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(window_config.unwrap_or_default()),
            ..default()
        }),
        CommonPlugins,
        #[cfg(feature = "wasm")]
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
            init_from_asset(&mut app, asset, skybox_id);
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

fn init_from_asset(app: &mut App, asset: InitAsset, skybox_id: String) {
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
                    app.add_systems(Startup, move |mut ew: EventWriter<MixamoImportEvent>| {
                        ew.send(MixamoImportEvent(anim.clone()));
                    });
                }
                _ => unreachable!(),
            }
        }
        InitAsset::ImportedScene(scene) => {
            app.add_systems(Startup, move |mut ew: EventWriter<ImportSceneEvent>| {
                ew.send(ImportSceneEvent {
                    path: scene.clone(),
                    name: None,
                });
            });
        }
    }
}
