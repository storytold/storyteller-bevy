use bevy::{math::vec3, prelude::*};
use bevy_mod_picking::prelude::*;

use crate::{interaction::TransformSpace, math::IsNonUniform};

#[derive(Component)]
pub(super) struct TransformGizmoRoot;

#[derive(Component)]
pub(super) struct TransformGizmo;

#[derive(Event)]
pub(super) struct RebuildGizmo;

#[derive(Bundle)]
pub(super) struct TransformGizmoBundle {
    name: Name,
    marker: TransformGizmo,
    spatial: SpatialBundle,
    scale: NaturalScale,
}

#[derive(Component, Deref)]
pub(super) struct NaturalScale(Vec3);

#[derive(Component, Deref)]
pub(super) struct InitialTransform(pub GlobalTransform);

impl TransformGizmo {
    pub fn compute_basis(owner_xform: &GlobalTransform, space: TransformSpace) -> Transform {
        let (inv_scale, inv_rotation, _) = owner_xform
            .affine()
            .inverse()
            .to_scale_rotation_translation();

        match space {
            TransformSpace::Local => {
                let inv_scale = if inv_scale.is_non_uniform() {
                    let s = inv_rotation * inv_scale;
                    vec3(s.x.abs(), s.y.abs(), s.z.abs())
                } else {
                    inv_scale
                };
                Transform::from_scale(inv_scale)
            }
            TransformSpace::World => Transform {
                rotation: inv_rotation,
                scale: inv_scale,
                ..default()
            },
        }
    }
}

impl TransformGizmoBundle {
    pub fn new(
        parent_xform: &GlobalTransform,
        space: TransformSpace,
        normalizing_scale: f32,
    ) -> Self {
        let mut basis = TransformGizmo::compute_basis(parent_xform, space);
        let nat_scale = basis.scale;
        basis.scale *= normalizing_scale;

        Self {
            name: Name::new("TransformGizmo"),
            marker: TransformGizmo,
            spatial: SpatialBundle::from_transform(basis),
            scale: NaturalScale(nat_scale),
        }
    }
}

pub(super) fn on_gizmo_element_hover(
    ev: Listener<Pointer<Over>>,
    mut ra_mats: ResMut<Assets<StandardMaterial>>,
    q_target: Query<&Handle<StandardMaterial>>,
) {
    let mat = ra_mats.get_mut(q_target.get(ev.target).unwrap()).unwrap();
    mat.base_color *= 2.;
}

pub(super) fn on_gizmo_element_unhover(
    ev: Listener<Pointer<Out>>,
    mut ra_mats: ResMut<Assets<StandardMaterial>>,
    q_target: Query<&Handle<StandardMaterial>>,
) {
    let mat = ra_mats.get_mut(q_target.get(ev.target).unwrap()).unwrap();
    mat.base_color *= 0.5;
}
