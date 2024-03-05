use bevy::{math::vec3, pbr::NotShadowCaster, prelude::*, render::view::RenderLayers};
use bevy_mod_picking::prelude::{Drag, DragEnd, DragStart, Listener, On, Out, Over, Pointer};

use crate::{interaction::TransformSpace, math::NearlyEq, MainCamera};

use super::{
    on_gizmo_element_hover, on_gizmo_element_unhover, GizmoMats, GizmoMeshes, InitialHitPoint,
    InitialTransform, TransformGizmo, TransformGizmoBundle, GIZMO_RENDER_LAYER,
};

#[derive(Component)]
pub(super) enum ScaleWidget {
    Axis(Vec3),
    Plane(Vec3),
    UniformScale,
}

#[inline]
pub(super) fn spawn_scale_gizmo(
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
        On::<Pointer<Over>>::run(on_gizmo_element_hover),
        On::<Pointer<Out>>::run(on_gizmo_element_unhover),
        On::<Pointer<DragStart>>::run(on_scale_start),
        On::<Pointer<Drag>>::run(on_scale),
        On::<Pointer<DragEnd>>::run(on_scale_end),
    ))
    .with_children(|cmd| {
        use super::{CIRC_45, ROT_X, ROT_Y, ROT_Z};

        // Center sphere
        cmd.spawn((
            NotShadowCaster,
            RenderLayers::layer(GIZMO_RENDER_LAYER),
            ScaleWidget::UniformScale,
            MaterialMeshBundle {
                mesh: meshes.uniform_scale_cube.clone(),
                material: mats.translate_ss.clone(),
                transform: Transform::IDENTITY,
                ..default()
            },
        ));

        // X
        cmd.spawn((
            NotShadowCaster,
            RenderLayers::layer(GIZMO_RENDER_LAYER),
            ScaleWidget::Axis(Vec3::X),
            MaterialMeshBundle {
                mesh: meshes.translate_line.clone(),
                material: mats.scale_x.clone(),
                transform: Transform {
                    rotation: ROT_X,
                    translation: vec3(0.6, 0., 0.),
                    ..default()
                },
                ..default()
            },
        ));
        cmd.spawn((
            NotShadowCaster,
            RenderLayers::layer(GIZMO_RENDER_LAYER),
            ScaleWidget::Axis(Vec3::X),
            MaterialMeshBundle {
                mesh: meshes.scale_cube.clone(),
                material: mats.scale_x.clone(),
                transform: Transform {
                    rotation: ROT_X,
                    translation: vec3(1., 0., 0.),
                    ..default()
                },
                ..default()
            },
        ));

        // Y
        cmd.spawn((
            NotShadowCaster,
            RenderLayers::layer(GIZMO_RENDER_LAYER),
            ScaleWidget::Axis(Vec3::Y),
            MaterialMeshBundle {
                mesh: meshes.translate_line.clone(),
                material: mats.scale_y.clone(),
                transform: Transform {
                    rotation: ROT_Y,
                    translation: vec3(0., 0.6, 0.),
                    ..default()
                },
                ..default()
            },
        ));
        cmd.spawn((
            NotShadowCaster,
            RenderLayers::layer(GIZMO_RENDER_LAYER),
            ScaleWidget::Axis(Vec3::Y),
            MaterialMeshBundle {
                mesh: meshes.scale_cube.clone(),
                material: mats.scale_y.clone(),
                transform: Transform {
                    rotation: ROT_Y,
                    translation: vec3(0., 1., 0.),
                    ..default()
                },
                ..default()
            },
        ));

        // Z
        cmd.spawn((
            NotShadowCaster,
            RenderLayers::layer(GIZMO_RENDER_LAYER),
            ScaleWidget::Axis(Vec3::Z),
            MaterialMeshBundle {
                mesh: meshes.translate_line.clone(),
                material: mats.scale_z.clone(),
                transform: Transform {
                    rotation: ROT_Z,
                    translation: vec3(0., 0., 0.6),
                    ..default()
                },
                ..default()
            },
        ));
        cmd.spawn((
            NotShadowCaster,
            RenderLayers::layer(GIZMO_RENDER_LAYER),
            ScaleWidget::Axis(Vec3::Z),
            MaterialMeshBundle {
                mesh: meshes.scale_cube.clone(),
                material: mats.scale_z.clone(),
                transform: Transform {
                    rotation: ROT_Z,
                    translation: vec3(0., 0., 1.),
                    ..default()
                },
                ..default()
            },
        ));

        // YZ Plane
        cmd.spawn((
            NotShadowCaster,
            RenderLayers::layer(GIZMO_RENDER_LAYER),
            ScaleWidget::Plane(Vec3::X),
            MaterialMeshBundle {
                mesh: meshes.translate_plane.clone(),
                material: mats.translate_yz.clone(),
                transform: Transform {
                    rotation: ROT_X,
                    translation: vec3(0., CIRC_45, CIRC_45),
                    ..default()
                },
                ..default()
            },
        ));
        // XZ Plane
        cmd.spawn((
            NotShadowCaster,
            RenderLayers::layer(GIZMO_RENDER_LAYER),
            ScaleWidget::Plane(Vec3::Y),
            MaterialMeshBundle {
                mesh: meshes.translate_plane.clone(),
                material: mats.translate_xz.clone(),
                transform: Transform {
                    rotation: ROT_Y,
                    translation: vec3(CIRC_45, 0., CIRC_45),
                    ..default()
                },
                ..default()
            },
        ));
        // XY Plane
        cmd.spawn((
            NotShadowCaster,
            RenderLayers::layer(GIZMO_RENDER_LAYER),
            ScaleWidget::Plane(Vec3::Z),
            MaterialMeshBundle {
                mesh: meshes.translate_plane.clone(),
                material: mats.translate_xy.clone(),
                transform: Transform {
                    rotation: ROT_Z,
                    translation: vec3(CIRC_45, CIRC_45, 0.),
                    ..default()
                },
                ..default()
            },
        ));
    });
}

pub(super) fn on_scale_start(
    event: Listener<Pointer<DragStart>>,
    mut cmd: Commands,
    q_xform: Query<(Entity, &GlobalTransform)>,
    q_gizmos: Query<&TransformGizmo>,
) {
    for gizmo_data in q_gizmos.iter() {
        if let Ok((ent, &world_xform)) = q_xform.get(gizmo_data.target) {
            let hit_point = event.hit.position.unwrap();

            cmd.entity(ent)
                .insert((InitialTransform(world_xform), InitialHitPoint(hit_point)));
        }
    }
}

pub(super) fn on_scale_end(
    _: Listener<Pointer<DragEnd>>,
    mut cmd: Commands,
    q_initial_transforms: Query<Entity, With<InitialTransform>>,
) {
    for ent in q_initial_transforms.iter() {
        cmd.entity(ent).remove::<InitialTransform>();
    }
}

pub(super) fn on_scale(
    event: Listener<Pointer<Drag>>,
    mut q_root: Query<(&mut Transform, &InitialTransform, &InitialHitPoint)>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    q_gizmo: Query<(&GlobalTransform, &TransformGizmo), Without<MainCamera>>,
    q_widget: Query<&ScaleWidget>,
) {
    let (gizmo_xform, gizmo_data) = q_gizmo.single();
    let Ok((mut target_transform, init_global_transform, initial_hit_point)) =
        q_root.get_mut(gizmo_data.target)
    else {
        error!("Scale gizmo target not found");
        return;
    };
    let (cam, cam_xform) = q_camera.single();

    // Find the initial origin point in screen-space and add the total screen-space drag distance
    let (initial_scale, _, origin_ws) = init_global_transform.to_scale_rotation_translation();
    let origin_ss = cam
        .world_to_viewport(cam_xform, **initial_hit_point)
        .unwrap();
    let target_ss = origin_ss + event.distance;

    // Create a camera-to-world ray that passes through the screen-space target point
    let ray = cam.viewport_to_world(cam_xform, target_ss).unwrap();

    use ScaleWidget::*;

    match q_widget.get(event.target) {
        Ok(Axis(axis)) => {
            // Transform the translation axis into world-space
            let axis_ws = gizmo_xform.affine().transform_vector3(*axis).normalize();

            // Find the plane vector that's orthogonal to the translation axis
            // and closest to parallel with the view direction
            let axis_tan = axis_ws.cross(cam_xform.back());
            let plane_vector = axis_ws.cross(axis_tan);

            // Find the intersection of the camera-to-world ray against that plane
            let Some(hit_dist) = ray.intersect_plane(origin_ws, plane_vector) else {
                return;
            };
            let hit_point = ray.origin + ray.direction * hit_dist;

            // Project the vector from origin -> hit point onto the translation
            // axis to constrain the translation
            let drag_vector = hit_point - origin_ws;
            let projection_length = drag_vector.dot(axis_ws);
            let target_ws = origin_ws + axis_ws * projection_length;

            let distance_from_initial_click = initial_hit_point.distance(origin_ws);
            let distance_from_current = target_ws.distance(origin_ws);

            let scale_factor = distance_from_current / distance_from_initial_click;

            info!("Scale Factor {scale_factor}");

            let (target_x, target_y, target_z) = initial_scale.into();
            let new_scale = if axis.nearly_eq(Vec3::Y) {
                Vec3::new(target_x, target_y * scale_factor, target_z)
            } else if axis.nearly_eq(Vec3::X) {
                Vec3::new(target_x * scale_factor, target_y, target_z)
            } else if axis.nearly_eq(Vec3::Z) {
                Vec3::new(target_x, target_y, target_z * scale_factor)
            } else {
                unreachable!();
            };

            info!("New Scale {:?}", new_scale);

            target_transform.scale = new_scale;
        }
        Ok(Plane(plane_normal)) => {
            let normal_ws = gizmo_xform
                .affine()
                .transform_vector3(*plane_normal)
                .normalize();
            let Some(hit_dist) = ray.intersect_plane(origin_ws, normal_ws) else {
                return;
            };
            let hit_point = ray.origin + ray.direction * hit_dist;

            let distance_from_initial_click = initial_hit_point.distance(origin_ws);
            let distance_from_current = hit_point.distance(origin_ws);

            let scale = distance_from_current / distance_from_initial_click;

            let scale_offset = if plane_normal.nearly_eq(Vec3::Z) {
                Vec3::new(scale, scale, 1.0)
            } else if plane_normal.nearly_eq(Vec3::Y) {
                Vec3::new(scale, 1.0, scale)
            } else if plane_normal.nearly_eq(Vec3::X) {
                Vec3::new(1.0, scale, scale)
            } else {
                unreachable!();
            };

            target_transform.scale = initial_scale * scale_offset;
        }
        Ok(UniformScale) => {
            let original_click_ss = cam
                .world_to_viewport(cam_xform, **initial_hit_point)
                .unwrap();

            let sign = if original_click_ss.y > target_ss.y {
                1.0
            } else {
                -1.0
            };
            let divisor = original_click_ss.y;
            let distance_click = original_click_ss.distance(target_ss);

            target_transform.scale =
                initial_scale + Vec3::splat((distance_click / divisor * sign) * 1.5);
        }
        _ => {}
    }
}
