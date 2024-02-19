use bevy::{
    math::{vec3, Affine3A, DVec3, Mat3A, Vec3A},
    pbr::NotShadowCaster,
    prelude::*,
    render::view::RenderLayers,
};
use bevy_mod_picking::prelude::*;
use bevy_mod_raycast::prelude::{Raycast, RaycastSettings};

use crate::{
    interaction::TransformSpace,
    math::{MatrixExtras, NearlyZero},
    MainCamera,
};

use super::{
    material::GizmoMaterial, on_gizmo_element_hover, on_gizmo_element_unhover, GizmoMats,
    GizmoMeshes, InitialTransform, TransformGizmo, TransformGizmoBundle, GIZMO_RENDER_LAYER,
};

#[derive(Component)]
pub(super) enum RotationWidget {
    Axis(Vec3),
    ScreenSpace,
    Trackball,
}

#[derive(Component, Deref)]
pub(super) struct InitialHitPoint(pub Vec3);

#[derive(Component, Deref, DerefMut)]
pub(super) struct LastHitPoint(pub Vec3);

#[derive(Component)]
pub(super) struct LastRotation {
    pub axis: Vec3,
    pub theta: f32,
}

#[inline]
pub(super) fn spawn_rotation_gizmo(
    cmd: &mut Commands,
    target: Entity,
    target_global_transform: &GlobalTransform,
    space: TransformSpace,
    normalizing_scale: f32,
    meshes: &GizmoMeshes,
    mats: &GizmoMats,
) {
    cmd.spawn((
        TransformGizmoBundle::new(target, target_global_transform, space, normalizing_scale),
        On::<Pointer<DragStart>>::run(on_rotation_start),
        On::<Pointer<Drag>>::run(on_rotate),
        On::<Pointer<DragEnd>>::run(on_rotation_end),
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
            let origin: Vec3A = world_xform.translation().into();
            let normal = (Vec3A::from(cam_xform.translation()) - origin).normalize();
            let (tangent, binormal) = normal.any_orthonormal_pair();
            let rotation = Mat3A::from_cols(tangent, normal, binormal);

            // Since we're overriding the local transform propagation, we need
            // to manually re-apply the normalization scale
            let scale = world_xform.affine().matrix3.decompose_scale();

            *world_xform = GlobalTransform::from(Affine3A {
                matrix3: rotation * scale,
                translation: origin,
            });
        }
    }
}

fn on_trackball_hover(
    ev: Listener<Pointer<Over>>,
    mut ra_mats: ResMut<Assets<GizmoMaterial>>,
    q_target: Query<&Handle<GizmoMaterial>>,
) {
    let mat = ra_mats.get_mut(q_target.get(ev.target).unwrap()).unwrap();
    mat.color.set_a(0.05);
}

fn on_trackball_unhover(
    ev: Listener<Pointer<Out>>,
    mut ra_mats: ResMut<Assets<GizmoMaterial>>,
    q_target: Query<&Handle<GizmoMaterial>>,
) {
    let mat = ra_mats.get_mut(q_target.get(ev.target).unwrap()).unwrap();
    mat.color.set_a(0.);
}

pub(super) fn on_rotation_start(
    event: Listener<Pointer<DragStart>>,
    mut cmd: Commands,
    q_xform: Query<(Entity, &GlobalTransform)>,
    q_gizmos: Query<&TransformGizmo>,
) {
    info!("Rotation start");

    let Ok(gizmo) = q_gizmos.get_single() else {
        error!("Rotation gizmo not found");
        return;
    };

    let Ok((ent, world_xform)) = q_xform.get(gizmo.target) else {
        error!("Rotation gizmo target not found");
        return;
    };

    let hit_point = event.hit.position.unwrap();

    cmd.entity(ent).insert((
        InitialTransform(*world_xform),
        InitialHitPoint(hit_point),
        LastHitPoint(hit_point),
        LastRotation {
            axis: Vec3::ZERO,
            theta: 0.,
        },
    ));
}

pub(super) fn on_rotation_end(
    _: Listener<Pointer<DragEnd>>,
    mut cmd: Commands,
    q_inited_components: Query<Entity, With<LastRotation>>,
) {
    info!("Rotation end");

    for ent in q_inited_components.iter() {
        cmd.entity(ent)
            .remove::<InitialTransform>()
            .remove::<InitialHitPoint>()
            .remove::<LastHitPoint>()
            .remove::<LastRotation>();
    }
}

pub(super) fn on_rotate(
    event: Listener<Pointer<Drag>>,
    mut raycast: Raycast,
    mut q_root: Query<(
        &mut Transform,
        Option<&Parent>,
        &InitialTransform,
        &InitialHitPoint,
        &mut LastHitPoint,
        &mut LastRotation,
    )>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    q_gizmo: Query<&GlobalTransform, (With<TransformGizmo>, Without<MainCamera>)>,
    q_world_xforms: Query<&GlobalTransform, (With<Children>, Without<MainCamera>)>,
    q_widget: Query<&RotationWidget>,
) {
    // Components from the gizmo root (the entity being transformed)
    let Ok((mut xform, parent, init_xform, init_point, mut last_point, mut last_rotation)) =
        q_root.get_single_mut()
    else {
        return;
    };
    let Ok(gizmo_xform) = q_gizmo.get_single() else {
        return;
    };
    let Ok((cam, cam_xform)) = q_camera.get_single() else {
        return;
    };

    // `Transform::rotate_axis` takes an axis vector relative to the transform's
    // _parent_. We'll use the gizmo root's parent's global transform (if it
    // exists) to move the world-space axis of rotation into the parent space
    // when applying the rotation.
    let identity_xform = GlobalTransform::default();
    let root_parent_xform = parent
        .and_then(|parent| q_world_xforms.get(parent.get()).ok())
        .unwrap_or(&identity_xform);

    // We compute the rotation by finding a triangle (in world-space) where:
    //   p0    = the gizmo/object origin
    //   p1,p2 = the intersections of a camera-to-world raycast, where p1 is the
    //           previous pointer location and p2 is the current.
    let p1 = **last_point;
    let p0 = init_xform.translation();

    // Find the screen-space p2 by adding the total drag distance to the pointer
    // location recorded on `DragStart`
    let p_init_ss = cam.world_to_viewport(cam_xform, **init_point).unwrap();
    let p2_ss = p_init_ss + event.distance;

    // Create a camera-to-world ray that passes through that point. In trackball
    // mode, we'll trace this against the surface of the trackball sphere to
    // find the world-space point. In all other modes, we'll trace against a
    // plane whose origin is p0 and whose normal is the axis of the "ring"
    // that's being dragged.
    let ray = cam.viewport_to_world(cam_xform, p2_ss).unwrap();

    use RotationWidget::*;

    match q_widget.get(event.target) {
        Ok(Trackball) => {
            // Raycast against the trackball sphere to find p2
            let filter = |ent: Entity| ent == event.target;
            let early_exit_test = |ent: Entity| ent != event.target;
            let settings = RaycastSettings {
                filter: &filter,
                early_exit_test: &early_exit_test,
                ..default()
            };
            let hits = raycast.cast_ray(ray.into(), &settings);

            // If the pointer is no longer in the vicinity of the sphere, we'll
            // invoke this closure to extrapolate additional rotation from the
            // last recorded axis and angle
            let mut extrapolate = || {
                use std::f32::consts::PI;

                let axis_parent_space = last_rotation.axis;
                if axis_parent_space.nearly_zero() {
                    return;
                }

                let view_plane_normal = (cam_xform.translation() - p1).normalize();
                let hit_dist = ray.intersect_plane(p1, view_plane_normal).unwrap();
                let p2 = ray.origin + ray.direction * hit_dist;
                let drag_dist = (p2 - p1).length();
                // TODO: Sensitivity could use some tuning
                let theta = last_rotation.theta + drag_dist * PI * 0.5;

                let mut init_xform_local = init_xform.reparented_to(root_parent_xform);
                init_xform_local.rotate_axis(axis_parent_space, theta);

                *xform = init_xform_local;
            };

            if hits.is_empty() {
                return extrapolate();
            }
            let (_, hit) = &hits[0];

            // Use 64-bit precision for all of these vectors to derive an
            // accurate axis of rotation even at very small theta values
            let p0 = DVec3::from(p0);
            let p1 = DVec3::from(**init_point);
            let p2 = DVec3::from(hit.position());

            let a = (p1 - p0).normalize_or_zero();
            let b = (p2 - p0).normalize_or_zero();
            if a.nearly_zero() || b.nearly_zero() {
                return extrapolate();
            }

            // Find the angle @ p0
            let theta = a.dot(b).acos() as f32;

            // Derive the axis of rotation by finding the vector perpendicular
            // to (p1-p0) and (p2-p0)
            let axis_ws = a.cross(b).normalize_or_zero();
            if axis_ws.nearly_zero() {
                return extrapolate();
            }

            // Transform the world-space axis into the parent space
            let axis_parent_space = root_parent_xform
                .affine()
                .inverse()
                .transform_vector3(vec3(axis_ws.x as f32, axis_ws.y as f32, axis_ws.z as f32))
                .normalize();

            // Apply the rotation and update our local state
            let mut init_xform_local = init_xform.reparented_to(root_parent_xform);
            init_xform_local.rotate_axis(axis_parent_space, theta);

            *xform = init_xform_local;
            **last_point = vec3(p2.x as f32, p2.y as f32, p2.z as f32);
            last_rotation.axis = axis_parent_space;
            last_rotation.theta = theta;
        }
        Ok(axis) => {
            // Find the world-space axis of rotation
            let axis_ws = match axis {
                ScreenSpace => (p0 - cam_xform.translation()).normalize(),
                Axis(normal) => gizmo_xform.affine().transform_vector3(*normal).normalize(),
                _ => unreachable!(),
            };

            // Find the world-space p2, if possible
            let Some(hit_dist) = ray.intersect_plane(p0, axis_ws) else {
                // This could fail if the axis of rotation is nearly
                // perpendicular to the view direction
                return;
            };
            let p2 = ray.origin + ray.direction * hit_dist;

            // Find the angle @ p0
            let a = p1 - p0;
            let b = p2 - p0;
            let theta = (a.dot(b) / (a.length() * b.length())).acos();
            if theta.is_nan() {
                return;
            }

            // Find the _direction_ of the rotation by comparing the canonical
            // axis of rotation with (p1-p0) x (p2-p0).
            //
            // In theory, we could just use that cross-product directly as our
            // axis of rotation, but floating-point errors at very small theta
            // values make that unfeasible in practice, so instead we just flip
            // the canonical axis when appropriate.
            let diff = axis_ws - a.cross(b).normalize();
            let axis_ws = if diff.length() > 1. {
                -axis_ws
            } else {
                axis_ws
            };

            // Transform the world-space axis into the parent space
            let axis_parent_space = root_parent_xform
                .affine()
                .inverse()
                .transform_vector3(axis_ws)
                .normalize();

            // Apply the rotation and update our local state
            xform.rotate_axis(axis_parent_space, theta);
            **last_point = p2;
        }
        _ => {}
    }
}
