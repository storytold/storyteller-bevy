use std::time::Duration;

use bevy::{
    app::{AppExit, ScheduleRunnerPlugin},
    prelude::*,
};
use space_editor::prelude::{EditorEvent, EditorPrefabPath};
use wgpu::Extent3d;

use crate::{
    bvh::{BvhAnimCompleteEvent, TestBvh},
    scene::SceneState,
    screenshot::{ScreenCapture, ScreenShotPlugin},
    BvhPlugin, CommonPlugins, LoadLibraryObjectEvent, LoadSceneEvent, MainCamera,
    MixamoImportEvent, MixamoPlugin, SkyboxId, StudioMode, WorldGridPlugin,
};

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

            app.add_systems(
                PostUpdate,
                check_for_bvh_capture_complete.run_if(in_state(SceneState::Active)),
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
            app.add_systems(
                PostUpdate,
                check_for_mixamo_capture_complete.run_if(in_state(SceneState::Active)),
            );

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
