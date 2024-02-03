use bevy::{prelude::*, transform::TransformSystem};

use crate::{camera_controller::CameraMotionSystem, MainCamera};

use self::{
    camera::{spawn_gizmo_camera, GizmoCamera},
    combo::spawn_combo_translation_rotation_gizmo,
    common::*,
    resources::{init_resources, GizmoMats, GizmoMeshes},
    rotation::{spawn_rotation_gizmo, update_ss_ring_rotation},
    translation::spawn_translation_gizmo,
};

use super::{Selection, TransformMode, TransformType};

pub mod camera;
mod combo;
mod common;
mod resources;
mod rotation;
mod translation;

const GIZMO_RENDER_LAYER: u8 = 1;

/// (sin|cos)(pi / 4)
const CIRC_45: f32 = 0.70710677;
/// Mat3 ((0 -1 0) (1 0 0) (0 0 1))
const ROT_X: Quat = Quat::from_xyzw(0., 0., CIRC_45, -CIRC_45);
/// Mat3 ((1 0 0) (0 1 0) (0 0 1))
const ROT_Y: Quat = Quat::IDENTITY;
/// Mat3 ((1 0 0) (0 0 1) (0 -1 0))
const ROT_Z: Quat = Quat::from_xyzw(CIRC_45, 0., 0., CIRC_45);

pub struct GizmoPlugin;

impl Plugin for GizmoPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TransformGizmo>();

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
                        .and_then(not(any_with_component::<TransformGizmo>())),
                ),
                (update_target_global_trasform, sync_gizmo_transform)
                    .chain()
                    .after(CameraMotionSystem),
                normalize_gizmo_scale.after(CameraMotionSystem),
            ),
        );

        //sync camera viewports
        app.add_systems(Update, camera::sync_gizmo_and_main_cameras);

        app.add_systems(
            PostUpdate,
            update_ss_ring_rotation.after(TransformSystem::TransformPropagate),
        );
    }
}

fn despawn_existing_gizmo(mut cmd: Commands, q_gizmos: Query<Entity, With<TransformGizmo>>) {
    for ent in q_gizmos.iter() {
        cmd.entity(ent).despawn_recursive();
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
    let target = **r_selection;
    let Ok(xform) = q_xform.get(target) else {
        return;
    };

    let Ok(cam_xform) = q_cam_xform.get_single() else {
        return;
    };
    let dist = Vec3::distance(cam_xform.translation, xform.translation());
    let normalizing_scale = dist / 15.;

    // TODO: Look into using the new one-shot systems feature here
    //       https://bevyengine.org/news/bevy-0-12/#one-shot-systems
    match r_xform_mode.ttype {
        TransformType::TRANSLATE => {
            spawn_translation_gizmo(
                &mut cmd,
                target,
                xform,
                r_xform_mode.space,
                normalizing_scale,
                &r_meshes,
                &r_mats,
            );
        }
        TransformType::ROTATE => {
            spawn_rotation_gizmo(
                &mut cmd,
                target,
                xform,
                r_xform_mode.space,
                normalizing_scale,
                &r_meshes,
                &r_mats,
            );
        }
        ttype if ttype.contains(TransformType::TRANSLATE | TransformType::ROTATE) => {
            spawn_combo_translation_rotation_gizmo(
                &mut cmd,
                target,
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

fn sync_gizmo_transform(
    mut cmd: Commands,
    mut q_gizmo: Query<(Entity, &mut Transform, &TransformGizmo)>,
    q_targets: Query<&GlobalTransform>,
) {
    for (ent, mut transform, gizmo) in q_gizmo.iter_mut() {
        if let Ok(target) = q_targets.get(gizmo.target) {
            match gizmo.transform_space {
                super::TransformSpace::Local => {
                    let target_transform = target.compute_transform();
                    transform.translation = target_transform.translation;
                    transform.rotation = target_transform.rotation;
                }
                super::TransformSpace::World => {
                    transform.translation = target.translation();
                }
            }
        } else {
            cmd.entity(ent).despawn_recursive();
        }
    }
}

fn update_target_global_trasform(
    mut q_gizmo: Query<&TransformGizmo>,
    mut q_targets: ParamSet<(TransformHelper,)>,
    mut q_targets_global: Query<&mut GlobalTransform>,
) {
    for gizmo in q_gizmo.iter_mut() {
        if let Ok(global_transform) = q_targets.p0().compute_global_transform(gizmo.target) {
            if let Ok(mut global) = q_targets_global.get_mut(gizmo.target) {
                *global = global_transform;
            }
        }
    }
}

fn normalize_gizmo_scale(
    q_cam_xform: Query<&Transform, With<MainCamera>>,
    mut q_gizmo: Query<
        (&mut Transform, &GlobalTransform),
        (With<TransformGizmo>, Without<MainCamera>),
    >,
) {
    let Ok(cam_xform) = q_cam_xform.get_single() else {
        return;
    };
    let Ok((mut xform, _)) = q_gizmo.get_single_mut() else {
        return;
    };

    let dist = Vec3::distance(cam_xform.translation, xform.translation);
    xform.scale = Vec3::splat(dist / 15.);
}
