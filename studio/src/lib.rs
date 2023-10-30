#![allow(clippy::type_complexity, clippy::too_many_arguments)]

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::DefaultPickingPlugins;
use camera_controller::{CameraController, CameraControllerPlugin, CameraMotionSystem};
use input::StudioInputPlugin;
use interaction::InteractionPlugin;
use math::InterpolationPlugin;
use scene::ScenePlugin;
use skybox::SkyboxPlugin;
use ui::UiLayerPlugin;
use viewport::ViewportSize;
use wasm_bindgen::prelude::*;

mod camera_controller;
mod input;
mod interaction;
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
        DefaultPickingPlugins,
        WorldInspectorPlugin::new(),
        InteractionPlugin,
        StudioInputPlugin,
        InterpolationPlugin,
        CameraControllerPlugin,
        ScenePlugin,
        SkyboxPlugin,
        UiLayerPlugin,
    ))
    .insert_resource(GizmoConfig {
        enabled: true,
        depth_bias: -0.1,
        line_perspective: false,
        line_width: 2.,
        ..default()
    })
    .add_systems(
        Update,
        (
            update_gizmo_line_thickness,
            draw_world_axes.after(CameraMotionSystem),
        ),
    )
    .run();
}

fn update_gizmo_line_thickness(
    mut config: ResMut<GizmoConfig>,
    q_window: Query<&Window, (Changed<Window>, With<PrimaryWindow>)>,
) {
    if let Ok(scale_factor) = q_window.get_single().map(|win| win.scale_factor() as f32) {
        config.line_width = 2. * scale_factor;
    }
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
