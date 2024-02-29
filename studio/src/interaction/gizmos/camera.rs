use bevy::{
    core_pipeline::{clear_color::ClearColorConfig, fxaa::Fxaa},
    prelude::*,
    render::view::RenderLayers,
};
use space_editor::space_editor_ui::NotShowCamera;

use crate::{
    anim::ik::{IkRigCamera, IK_RIG_RENDER_LAYER},
    MainCamera,
};

use super::GIZMO_RENDER_LAYER;

#[derive(Component)]
pub struct GizmoCamera;

pub(super) fn spawn_gizmo_camera(
    mut cmd: Commands,
    q_main_camera: Query<Entity, With<MainCamera>>,
) {
    let ent = q_main_camera.single();

    cmd.entity(ent).with_children(|cmd| {
        // TODO: This doesn't really belong here, but the render pipeline seems
        //       to break if we don't spawn these cameras in the correct order.
        //       We should probably add a dedicated module/plugin for setting up
        //       the editor cameras.
        cmd.spawn((
            Name::new("IK Rig Camera"),
            IkRigCamera,
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
            RenderLayers::layer(IK_RIG_RENDER_LAYER),
            Fxaa::default(),
            NotShowCamera,
        ));

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
                    order: 2,
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

pub(super) fn sync_main_and_aux_cameras(
    q_main_camera: Query<(Entity, &Camera), With<MainCamera>>,
    mut q_other_cameras: Query<&mut Camera, Without<MainCamera>>,
    q_children: Query<&Children>,
) {
    let Ok((main_cam_ent, main_cam)) = q_main_camera.get_single() else {
        return;
    };

    for child in q_children.iter_descendants(main_cam_ent) {
        if let Ok(mut cam) = q_other_cameras.get_mut(child) {
            cam.viewport = main_cam.viewport.clone();
        }
    }
}
