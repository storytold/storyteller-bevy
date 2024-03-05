use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::interaction::TransformSpace;

use super::material::GizmoMaterial;

#[derive(Component, Reflect)]
pub struct TransformGizmo {
    pub(super) target: Entity,
    pub(super) transform_space: TransformSpace,
}

#[derive(Bundle)]
pub(super) struct TransformGizmoBundle {
    name: Name,
    marker: TransformGizmo,
    spatial: SpatialBundle,
}

#[derive(Component, Deref)]
pub(super) struct InitialTransform(pub GlobalTransform);

impl TransformGizmoBundle {
    pub fn new(
        target: Entity,
        target_global_transform: &GlobalTransform,
        space: TransformSpace,
        normalizing_scale: f32,
    ) -> Self {
        let name = Name::new("TransformGizmo");
        let marker = TransformGizmo {
            target,
            transform_space: space,
        };

        match space {
            TransformSpace::Local => {
                let mut transform = target_global_transform.compute_transform();
                transform.scale = Vec3::splat(normalizing_scale);

                Self {
                    name,
                    marker,
                    spatial: SpatialBundle {
                        global_transform: GlobalTransform::from(transform),
                        transform,
                        ..default()
                    },
                }
            }
            TransformSpace::World => {
                let transform = Transform::from_translation(target_global_transform.translation())
                    .with_scale(Vec3::splat(normalizing_scale));

                Self {
                    name,
                    marker,
                    spatial: SpatialBundle {
                        global_transform: GlobalTransform::from(transform),
                        transform,
                        ..default()
                    },
                }
            }
        }
    }
}

pub(super) fn on_gizmo_element_hover(
    ev: Listener<Pointer<Over>>,
    mut ra_mats: ResMut<Assets<GizmoMaterial>>,
    q_target: Query<&Handle<GizmoMaterial>>,
) {
    let mat = ra_mats.get_mut(q_target.get(ev.target).unwrap()).unwrap();
    mat.color *= 2.;
}

pub(super) fn on_gizmo_element_unhover(
    ev: Listener<Pointer<Out>>,
    mut ra_mats: ResMut<Assets<GizmoMaterial>>,
    q_target: Query<&Handle<GizmoMaterial>>,
) {
    let mat = ra_mats.get_mut(q_target.get(ev.target).unwrap()).unwrap();
    mat.color *= 0.5;
}

#[derive(Component, Deref)]
pub(super) struct InitialHitPoint(pub Vec3);
