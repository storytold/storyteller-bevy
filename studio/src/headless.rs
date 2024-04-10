use std::time::Duration;

use bevy::{
    app::{AppExit, ScheduleRunnerPlugin},
    asset::AssetMetaCheck,
    core_pipeline::Skybox,
    prelude::*,
    winit::WinitPlugin,
};
use bevy_web_asset::WebAssetPlugin;
use space_editor::prelude::{EditorEvent, EditorPrefabPath};
use wgpu::Extent3d;

use crate::{
    anim::timeline_animation::{Keyframe, TimelineInteractionEvent},
    bvh::{BvhAnimCompleteEvent, TestBvh},
    scene::{SceneState, SkyboxHolder},
    screenshot::{ScreenCapture, ScreenShotPlugin},
    BvhPlugin, CommonPlugins, LoadLibraryObjectEvent, LoadSceneEvent, MainCamera,
    MixamoImportEvent, MixamoPlugin, RemoteAssetPlugin, SkyboxId, StudioMode, WorldGridPlugin,
};

#[derive(Debug, Resource, Clone)]
pub enum HeadlessMode {
    Bvh(String, String),
    Scene(String, String),
    Mixamo(String, String),
    None,
}

#[derive(Resource)]
pub enum CameraMotion {
    Orbit(f32),
    Pan(f32),
    Zoom(f32),
    Static,
}

pub fn start_headless(
    mode: HeadlessMode,
    fps: u32,
    outout_dir: &str,
    frames_to_capture: Option<u32>,
    image_res: Extent3d,
) -> App {
    let mut app = App::new();

    app.insert_resource(mode.clone());
    app.insert_resource(StudioMode::Headless);
    app.insert_resource(HeadlessSettings {
        max_frames: frames_to_capture,
        output_dims: image_res,
        output_dir: outout_dir.to_string(),
        fps: fps as f32,
        frame_delta: 1.0 / (fps as f32),
    });

    app.insert_resource(AssetMetaCheck::Never);

    app.add_plugins((
        ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / fps as f64)),
        WebAssetPlugin,
        RemoteAssetPlugin,
        DefaultPlugins
            .set(TaskPoolPlugin {
                task_pool_options: TaskPoolOptions {
                    async_compute: bevy::core::TaskPoolThreadAssignmentPolicy {
                        min_threads: 1,
                        max_threads: usize::MAX,
                        percent: 0.75,
                    },
                    ..default()
                },
            })
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: bevy::window::ExitCondition::DontExit,
                close_when_requested: true,
            })
            .set(AssetPlugin {
                mode: AssetMode::Unprocessed,
                ..default()
            })
            .disable::<WinitPlugin>(),
        ScreenShotPlugin,
        CommonPlugins
            .build()
            .disable::<WorldGridPlugin>()
            .disable::<crate::UiLayerPlugin>(),
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
                move |mut cmd: Commands,
                      mut ew: EventWriter<EditorEvent>,
                      asset_server: Res<AssetServer>| {
                    cmd.insert_resource(SkyboxHolder(Some(Skybox {
                        image: asset_server.load(skybox.clone()),
                        brightness: 1.0,
                    })));

                    ew.send(EditorEvent::Load(EditorPrefabPath::File(scene.clone())));
                },
            );
            app.add_systems(OnEnter(SceneState::Active), start_timeline_animation);
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

#[derive(Resource)]
pub struct HeadlessSettings {
    pub output_dims: Extent3d,
    pub output_dir: String,
    pub max_frames: Option<u32>,
    pub fps: f32,
    pub frame_delta: f32,
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
            max_frames: None,
            fps: 24.0,
            frame_delta: 1.0 / 24.0,
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

#[cfg(not(feature = "wasm"))]
fn start_timeline_animation(
    mut headless_settings: ResMut<HeadlessSettings>,
    keyframes: Query<&Keyframe>,
    camera: Query<Entity, With<MainCamera>>,
    mut commands: Commands,
    mut ev: EventWriter<TimelineInteractionEvent>,
) {
    if keyframes.is_empty() {
        let camera = camera.single();
        commands.entity(camera).insert(
            ScreenCapture::from_out(&headless_settings.output_dir)
                //Part of quick hack, see screenshot.rs#261
                .with_max_frames(headless_settings.max_frames.unwrap_or(0) + 3),
        );
        if headless_settings.max_frames.is_none() {
            error!("Both keyframes and max_frames are empty. Aborting.");
        }
        return;
    }
    commands.remove_resource::<CameraMotion>();
    let total_time = keyframes
        .iter()
        .fold(1f32, |previous, keyframe| previous.max(keyframe.time))
        .max(3.0);

    let mut frames = (total_time * headless_settings.fps).ceil() as u32;
    if let Some(max_frames) = headless_settings.max_frames {
        frames = max_frames;
    }
    info!("Frames to capture {}", frames);
    headless_settings.max_frames = Some(frames);
    ev.send(TimelineInteractionEvent::Play);
    let camera = camera.single();
    commands.entity(camera).insert(
        ScreenCapture::from_out(&headless_settings.output_dir)
            //Part of quick hack, see screenshot.rs#261
            .with_max_frames(frames + 3),
    );
}
