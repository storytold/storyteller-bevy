use bevy::prelude::*;

use crate::MainCamera;

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
    r_meshes: Res<GizmoMeshes>,
    r_mats: Res<GizmoMats>,
    q_xform: Query<&GlobalTransform>,
) {
    let ent = **r_selection;
    let Ok(xform) = q_xform.get(ent) else {
        return;
    };

    match r_xform_mode.ttype {
        TransformType::TRANSLATE => {
            spawn_translation_gizmo(&mut cmd, ent, xform, r_xform_mode.space, &r_meshes, &r_mats);
        }
        _ => {
            // TODO
        }
    }
}
