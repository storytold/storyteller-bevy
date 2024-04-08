use bevy::{core_pipeline::fxaa::Fxaa, prelude::*, render::view::RenderLayers};
use space_editor::space_editor_ui::camera_plugin::NotShowCamera;

use crate::MainCamera;

use super::GIZMO_RENDER_LAYER;

#[derive(Component)]
pub struct GizmoCamera;

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
                camera_3d: Camera3d { ..default() },
                camera: Camera {
                    hdr: true,
                    order: 1,
                    clear_color: ClearColorConfig::None,
                    ..default()
                },
                ..default()
            },
            RenderLayers::layer(GIZMO_RENDER_LAYER),
            Fxaa::default(),
            NotShowCamera,
        ));
    });
}

pub(super) fn sync_gizmo_and_main_cameras(
    q_main_camera: Query<&Camera, (With<MainCamera>, Without<GizmoCamera>)>,
    mut q_gizmo_camera: Query<&mut Camera, (With<GizmoCamera>, Without<MainCamera>)>,
) {
    let Ok(main_cam) = q_main_camera.get_single() else {
        return;
    };

    let Ok(mut gizmo_cam) = q_gizmo_camera.get_single_mut() else {
        return;
    };

    gizmo_cam.viewport = main_cam.viewport.clone();
}
