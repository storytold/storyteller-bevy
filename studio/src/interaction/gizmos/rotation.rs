use bevy::{
    ecs::system::EntityCommands,
    math::{vec3, Affine3A},
    pbr::NotShadowCaster,
    prelude::*,
    render::view::RenderLayers,
};
use bevy_mod_picking::prelude::*;

use crate::{interaction::TransformSpace, MainCamera};

use super::{
    common::InitialTransform, on_gizmo_element_hover, on_gizmo_element_unhover, GizmoMats,
    GizmoMeshes, TransformGizmoBundle, TransformGizmoRoot, GIZMO_RENDER_LAYER,
};

#[derive(Component)]
pub(super) enum RotationWidget {
    Axis(Vec3),
    ScreenSpace,
    Trackball,
}

#[derive(Component)]
pub(super) struct ScreenSpaceRotationRing;

#[derive(Component, Deref)]
struct InitialHitPoint(Vec3);

#[inline]
pub(super) fn spawn_rotation_gizmo(
    cmd: &mut Commands,
    ent: Entity,
    xform: &GlobalTransform,
    space: TransformSpace,
    normalizing_scale: f32,
    meshes: &GizmoMeshes,
    mats: &GizmoMats,
) {
    cmd.entity(ent)
        .insert(TransformGizmoRoot)
        .with_children(|cmd| {
            cmd.spawn((
                TransformGizmoBundle::new(xform, space, normalizing_scale),
                On::<Pointer<DragStart>>::run(on_drag_start),
                On::<Pointer<Drag>>::run(on_rotate),
                On::<Pointer<DragEnd>>::target_commands_mut(on_drag_end),
            ))
            .with_children(|cmd| {
                use super::{ROT_X, ROT_Y, ROT_Z};

                // Trackball
                cmd.spawn((
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                    RotationWidget::Trackball,
                    MaterialMeshBundle {
                        mesh: meshes.rotate_ball.clone(),
                        material: mats.rotate_trackball.clone(),
                        ..default()
                    },
                    On::<Pointer<Over>>::run(on_trackball_hover),
                    On::<Pointer<Out>>::run(on_trackball_unhover),
                ));

                // Screen-space
                cmd.spawn((
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                    RotationWidget::ScreenSpace,
                    MaterialMeshBundle {
                        mesh: meshes.rotate_ss.clone(),
                        material: mats.rotate_ss.clone(),
                        ..default()
                    },
                    On::<Pointer<Over>>::run(on_gizmo_element_hover),
                    On::<Pointer<Out>>::run(on_gizmo_element_unhover),
                ));

                // X
                cmd.spawn((
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                    RotationWidget::Axis(Vec3::X),
                    MaterialMeshBundle {
                        mesh: meshes.rotate_ring.clone(),
                        material: mats.rotate_x.clone(),
                        transform: Transform::from_rotation(ROT_X),
                        ..default()
                    },
                    On::<Pointer<Over>>::run(on_gizmo_element_hover),
                    On::<Pointer<Out>>::run(on_gizmo_element_unhover),
                ));
                // Y
                cmd.spawn((
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                    RotationWidget::Axis(Vec3::Y),
                    MaterialMeshBundle {
                        mesh: meshes.rotate_ring.clone(),
                        material: mats.rotate_y.clone(),
                        transform: Transform::from_rotation(ROT_Y),
                        ..default()
                    },
                    On::<Pointer<Over>>::run(on_gizmo_element_hover),
                    On::<Pointer<Out>>::run(on_gizmo_element_unhover),
                ));
                // Z
                cmd.spawn((
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                    RotationWidget::Axis(Vec3::Z),
                    MaterialMeshBundle {
                        mesh: meshes.rotate_ring.clone(),
                        material: mats.rotate_z.clone(),
                        transform: Transform::from_rotation(ROT_Z),
                        ..default()
                    },
                    On::<Pointer<Over>>::run(on_gizmo_element_hover),
                    On::<Pointer<Out>>::run(on_gizmo_element_unhover),
                ));
            });
        });
}

pub(super) fn update_ss_ring_rotation(
    q_camera: Query<&GlobalTransform, With<MainCamera>>,
    mut q_ring: Query<(&mut GlobalTransform, &RotationWidget), Without<MainCamera>>,
) {
    let Ok(cam_xform) = q_camera.get_single() else {
        return;
    };

    for (mut world_xform, widget) in q_ring.iter_mut() {
        if matches!(*widget, RotationWidget::ScreenSpace) {
            // Find basis vector to rotate the flat side of the ring toward the camera
            let origin = world_xform.translation();
            let normal = (cam_xform.translation() - origin).normalize();
            let (tangent, binormal) = normal.any_orthonormal_pair();
            let rotation = Mat3::from_cols(tangent, normal, binormal);

            // Since we're overriding the local transform propagation, we need
            // to manually re-apply the normalization scale
            let mat3 = world_xform.affine().matrix3;
            let scale = vec3(
                mat3.x_axis.length() * f32::signum(mat3.determinant()),
                mat3.y_axis.length(),
                mat3.z_axis.length(),
            );
            let scale = Mat3::from_cols(
                vec3(scale.x, 0., 0.),
                vec3(0., scale.y, 0.),
                vec3(0., 0., scale.z),
            );

            *world_xform = GlobalTransform::from(Affine3A {
                matrix3: (rotation * scale).into(),
                translation: origin.into(),
            });
        }
    }
}

fn on_trackball_hover(
    ev: Listener<Pointer<Over>>,
    mut ra_mats: ResMut<Assets<StandardMaterial>>,
    q_target: Query<&Handle<StandardMaterial>>,
) {
    let mat = ra_mats.get_mut(q_target.get(ev.target).unwrap()).unwrap();
    mat.base_color.set_a(0.2);
}

fn on_trackball_unhover(
    ev: Listener<Pointer<Out>>,
    mut ra_mats: ResMut<Assets<StandardMaterial>>,
    q_target: Query<&Handle<StandardMaterial>>,
) {
    let mat = ra_mats.get_mut(q_target.get(ev.target).unwrap()).unwrap();
    mat.base_color.set_a(0.);
}

fn on_drag_start(
    event: Listener<Pointer<DragStart>>,
    mut cmd: Commands,
    q_xform: Query<(Entity, &GlobalTransform), With<TransformGizmoRoot>>,
) {
    if let Ok((ent, &xform)) = q_xform.get_single() {
        cmd.entity(ent).insert((
            InitialTransform(xform),
            InitialHitPoint(event.hit.position.unwrap()),
        ));
    }
}

// TODO
fn on_rotate() {}

fn on_drag_end(_: &ListenerInput<Pointer<DragEnd>>, cmd: &mut EntityCommands<'_, '_, '_>) {
    cmd.remove::<InitialTransform>();
    cmd.remove::<InitialHitPoint>();
}
