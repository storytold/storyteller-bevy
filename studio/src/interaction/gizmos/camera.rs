use bevy::{
    core_pipeline::{clear_color::ClearColorConfig, fxaa::Fxaa},
    prelude::*,
    render::view::RenderLayers,
};

use crate::MainCamera;

use super::GIZMO_RENDER_LAYER;

#[derive(Resource, Deref, DerefMut)]
pub(super) struct GizmoCamera(Entity);

pub(super) fn spawn_gizmo_camera(
    mut cmd: Commands,
    q_cam_controller: Query<Entity, With<MainCamera>>,
) {
    if let Ok(ent) = q_cam_controller.get_single() {
        let mut gizmo_cam = None;

        cmd.entity(ent).with_children(|cmd| {
            gizmo_cam = Some(
                cmd.spawn((
                    Name::new("Gizmo Camera"),
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
                ))
                .id(),
            );
        });

        if let Some(ent) = gizmo_cam {
            cmd.insert_resource(GizmoCamera(ent));
        }
    }
}
