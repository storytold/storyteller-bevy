#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use input::StudioInputPlugin;
use orbit_cam::{CameraMotionSystem, OrbitCam, OrbitCamPlugin};
use skybox::SkyboxPlugin;
use test_scene::TestScenePlugin;
use ui::UiLayerPlugin;
use viewport::ViewportSize;
use wasm_bindgen::prelude::*;

mod input;
mod orbit_cam;
mod skybox;
mod test_scene;
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
                ..default()
            }),
            ..default()
        }),
        StudioInputPlugin,
        OrbitCamPlugin,
        TestScenePlugin,
        SkyboxPlugin,
        UiLayerPlugin,
    ))
    .add_systems(Update, draw_world_axes.after(CameraMotionSystem))
    .run();
}

fn draw_world_axes(mut gizmos: Gizmos, q: Query<&OrbitCam>) {
    let &OrbitCam { focus, radius, .. } = q.single();

    let mut draw_axis = |direction: Vec3, color: Color| {
        let end = focus + direction * radius / 15.;
        gizmos.line(focus, end, color);
    };

    draw_axis(Vec3::X, Color::RED);
    draw_axis(Vec3::Y, Color::GREEN);
    draw_axis(Vec3::Z, Color::BLUE);
}
