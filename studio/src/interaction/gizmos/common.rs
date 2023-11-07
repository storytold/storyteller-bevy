use bevy::{math::Affine3A, prelude::*};
use bevy_mod_picking::prelude::*;

use crate::{
    interaction::TransformSpace,
    math::{IsNonUniform, MatrixExtras, VecExtras},
};

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

#[derive(Component, Deref, Reflect)]
pub(super) struct NaturalScale(pub Vec3);

#[derive(Component, Deref)]
pub(super) struct InitialTransform(pub GlobalTransform);

impl TransformGizmoBundle {
    pub fn new(
        parent_xform: &GlobalTransform,
        space: TransformSpace,
        normalizing_scale: f32,
    ) -> Self {
        let name = Name::new("TransformGizmo");
        let marker = TransformGizmo;

        let Affine3A {
            matrix3,
            translation,
        } = parent_xform.affine();

        let (scale, rotation) = matrix3.decompose_scale_rotation();
        let inv_scale = scale.inverse();
        let nat_scale = NaturalScale(Vec3::from_scale_mat(inv_scale));

        match space {
            TransformSpace::Local => {
                let global_transform = GlobalTransform::from(Affine3A {
                    matrix3: rotation,
                    translation,
                });

                let transform = Transform::from_scale(*nat_scale * normalizing_scale);

                Self {
                    name,
                    marker,
                    spatial: SpatialBundle {
                        global_transform,
                        transform,
                        ..default()
                    },
                    scale: nat_scale,
                }
            }
            TransformSpace::World => {
                // FIXME: I'm struggling to figure out how to eliminate the
                //        "shear" from a fully inverted local transform when the
                //        object is non-uniformly scaled. As a workaround, we'll
                //        force these objects to operate in local-space mode.
                // TODO: If this turns into a long-term solution, update the UI
                //       to reflect that world-space transformation is not
                //       possible for this object (e.g., by disabling the
                //       world-space toggle button, maybe with an explanatory
                //       tooltip).
                let local_matrix = if nat_scale.is_non_uniform() {
                    inv_scale
                } else {
                    matrix3.inverse()
                };

                let global_transform = GlobalTransform::from(Affine3A {
                    matrix3,
                    translation,
                });

                let mat4 = Mat4::from_mat3a(local_matrix * normalizing_scale);
                let transform = Transform::from_matrix(mat4);

                Self {
                    name,
                    marker,
                    spatial: SpatialBundle {
                        global_transform,
                        transform,
                        ..default()
                    },
                    scale: nat_scale,
                }
            }
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
