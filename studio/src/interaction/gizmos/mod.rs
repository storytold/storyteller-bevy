use bevy::prelude::*;

use crate::{camera_controller::CameraMotionSystem, MainCamera};

use self::{
    camera::{spawn_gizmo_camera, GizmoCamera},
    common::*,
    resources::{init_resources, GizmoMats, GizmoMeshes},
    translation::spawn_translation_gizmo,
};

use super::{Selection, TransformMode, TransformType};

mod camera;
mod common;
mod resources;
mod translation;

const GIZMO_RENDER_LAYER: u8 = 1;

pub struct GizmoPlugin;

impl Plugin for GizmoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_resources);
        app.add_systems(
            PreUpdate,
            spawn_gizmo_camera.run_if(
                any_with_component::<MainCamera>()
                    .and_then(not(any_with_component::<GizmoCamera>())),
            ),
        );
        app.add_systems(
            Update,
            (
                despawn_existing_gizmo.run_if(
                    resource_changed_or_removed::<Selection>()
                        .or_else(resource_changed::<TransformMode>()),
                ),
                spawn_gizmo.after(despawn_existing_gizmo).run_if(
                    resource_exists::<Selection>()
                        .and_then(not(any_with_component::<TransformGizmoRoot>()))
                        .and_then(
                            resource_changed::<Selection>()
                                .or_else(resource_changed::<TransformMode>()),
                        ),
                ),
                normalize_gizmo_scale.after(CameraMotionSystem),
            ),
        );
    }
}

fn despawn_existing_gizmo(
    mut cmd: Commands,
    q_gizmo_root: Query<(Entity, &Children), With<TransformGizmoRoot>>,
    q_gizmos: Query<Entity, With<TransformGizmo>>,
) {
    if let Ok((ent, children)) = q_gizmo_root.get_single() {
        if let Some(child) = children.iter().find_map(|ent| q_gizmos.get(*ent).ok()) {
            cmd.entity(child).despawn_recursive();
        }

        cmd.entity(ent).remove::<TransformGizmoRoot>();
    }
}

fn spawn_gizmo(
    mut cmd: Commands,
    r_selection: Res<Selection>,
    r_xform_mode: Res<TransformMode>,
    q_xform: Query<&GlobalTransform>,
    q_cam_xform: Query<&Transform, With<MainCamera>>,
    r_meshes: Res<GizmoMeshes>,
    r_mats: Res<GizmoMats>,
) {
    let ent = **r_selection;
    let Ok(xform) = q_xform.get(ent) else {
        return;
    };

    let Ok(cam_xform) = q_cam_xform.get_single() else {
        return;
    };
    let dist = Vec3::distance(cam_xform.translation, xform.translation());
    let normalizing_scale = dist / 15.;

    match r_xform_mode.ttype {
        TransformType::TRANSLATE => {
            spawn_translation_gizmo(
                &mut cmd,
                ent,
                xform,
                r_xform_mode.space,
                normalizing_scale,
                &r_meshes,
                &r_mats,
            );
        }
        _ => {
            // TODO
        }
    }
}

fn normalize_gizmo_scale(
    q_cam_xform: Query<&Transform, With<MainCamera>>,
    mut q_gizmo: Query<
        (&mut Transform, &GlobalTransform, &NaturalScale),
        (With<TransformGizmo>, Without<MainCamera>),
    >,
) {
    let Ok(cam_xform) = q_cam_xform.get_single() else {
        return;
    };
    let Ok((mut xform, world_xform, nat_scale)) = q_gizmo.get_single_mut() else {
        return;
    };

    let dist = Vec3::distance(cam_xform.translation, world_xform.translation());
    xform.scale = **nat_scale * (dist / 15.);
}
