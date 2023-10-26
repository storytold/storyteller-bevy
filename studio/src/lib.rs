#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use camera_controller::{CameraController, CameraControllerPlugin, CameraMotionSystem};
use input::StudioInputPlugin;
use math::InterpolationPlugin;
use scene::ScenePlugin;
use skybox::SkyboxPlugin;
use ui::UiLayerPlugin;
use viewport::ViewportSize;
use wasm_bindgen::prelude::*;

mod camera_controller;
mod input;
mod math;
mod scene;
mod skybox;
mod ui;
mod viewport;

#[wasm_bindgen]
pub fn init(viewport_size: ViewportSize) {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let mut app = App::new();

    #[cfg(target_arch = "wasm32")]
    app.add_plugins(viewport::wasm::ViewportPlugin);

    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: viewport_size.into(),
                title: "Storyteller Studio".into(),
                ..default()
            }),
            ..default()
        }),
        StudioInputPlugin,
        InterpolationPlugin,
        CameraControllerPlugin,
        ScenePlugin,
        SkyboxPlugin,
        UiLayerPlugin,
    ))
    .add_systems(Update, draw_world_axes.after(CameraMotionSystem))
    .run();
}

fn draw_world_axes(mut gizmos: Gizmos, q: Query<&CameraController>) {
    let &CameraController { focus, radius, .. } = q.single();

    let mut draw_axis = |direction: Vec3, color: Color| {
        let end = focus + direction * radius / 15.;
        gizmos.line(focus, end, color);
    };

    draw_axis(Vec3::X, Color::RED);
    draw_axis(Vec3::Y, Color::GREEN);
    draw_axis(Vec3::Z, Color::BLUE);
}
