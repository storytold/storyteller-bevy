use bevy::{
    core_pipeline::{clear_color::ClearColorConfig, fxaa::Fxaa},
    prelude::*,
    render::view::RenderLayers,
};

use crate::MainCamera;

use super::GIZMO_RENDER_LAYER;

#[derive(Component)]
pub(super) struct GizmoCamera;

pub(super) fn spawn_gizmo_camera(
    mut cmd: Commands,
    q_main_camera: Query<Entity, With<MainCamera>>,
) {
    let ent = q_main_camera.single();

    cmd.entity(ent).with_children(|cmd| {
        cmd.spawn((
            Name::new("Gizmo Camera"),
            GizmoCamera,
            Camera3dBundle {
                camera_3d: Camera3d {
                    clear_color: ClearColorConfig::None,
                    ..default()
                },
                camera: Camera {
                    hdr: true,
                    order: 1,
                    ..default()
                },
                ..default()
            },
            RenderLayers::layer(GIZMO_RENDER_LAYER),
            Fxaa::default(),
        ));
    });
}
