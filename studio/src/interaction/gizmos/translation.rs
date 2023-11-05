use bevy::{math::vec3, pbr::NotShadowCaster, prelude::*, render::view::RenderLayers};
use bevy_mod_picking::prelude::{Drag, DragEnd, DragStart, Listener, On, Out, Over, Pointer};

use crate::{interaction::TransformSpace, MainCamera};

use super::{
    on_drag_end, on_drag_start, on_gizmo_element_hover, on_gizmo_element_unhover, GizmoMats,
    GizmoMeshes, InitialTransform, TransformGizmo, TransformGizmoBundle, TransformGizmoRoot,
    GIZMO_RENDER_LAYER,
};

#[derive(Component)]
enum TranslationWidget {
    Axis(Vec3),
    Plane(Vec3),
    ScreenSpace,
}

#[inline]
pub(super) fn spawn_translation_gizmo(
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
                On::<Pointer<Over>>::run(on_gizmo_element_hover),
                On::<Pointer<Out>>::run(on_gizmo_element_unhover),
                On::<Pointer<DragStart>>::run(on_drag_start),
                On::<Pointer<Drag>>::run(on_translate),
                On::<Pointer<DragEnd>>::target_commands_mut(on_drag_end),
            ))
            .with_children(|cmd| {
                let rot_x = Quat::from_mat3(&Mat3::from_cols(Vec3::NEG_Y, Vec3::X, Vec3::Z));
                let rot_y = Quat::IDENTITY;
                let rot_z = Quat::from_mat3(&Mat3::from_cols(Vec3::X, Vec3::Z, Vec3::NEG_Y));

                // Center sphere
                cmd.spawn((
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                    TranslationWidget::ScreenSpace,
                    MaterialMeshBundle {
                        mesh: meshes.translate_ss.clone(),
                        material: mats.translate_ss.clone(),
                        transform: Transform::IDENTITY,
                        ..default()
                    },
                ));

                // X
                cmd.spawn((
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                    TranslationWidget::Axis(Vec3::X),
                    MaterialMeshBundle {
                        mesh: meshes.translate_line.clone(),
                        material: mats.translate_x.clone(),
                        transform: Transform {
                            rotation: rot_x,
                            translation: vec3(0.6, 0., 0.),
                            scale: Vec3::splat(1.),
                        },
                        ..default()
                    },
                ));
                cmd.spawn((
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                    TranslationWidget::Axis(Vec3::X),
                    MaterialMeshBundle {
                        mesh: meshes.translate_arrow.clone(),
                        material: mats.translate_x.clone(),
                        transform: Transform {
                            rotation: rot_x,
                            translation: vec3(1., 0., 0.),
                            scale: Vec3::splat(1.),
                        },
                        ..default()
                    },
                ));

                // Y
                cmd.spawn((
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                    TranslationWidget::Axis(Vec3::Y),
                    MaterialMeshBundle {
                        mesh: meshes.translate_line.clone(),
                        material: mats.translate_y.clone(),
                        transform: Transform {
                            rotation: rot_y,
                            translation: vec3(0., 0.6, 0.),
                            scale: Vec3::splat(1.),
                        },
                        ..default()
                    },
                ));
                cmd.spawn((
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                    TranslationWidget::Axis(Vec3::Y),
                    MaterialMeshBundle {
                        mesh: meshes.translate_arrow.clone(),
                        material: mats.translate_y.clone(),
                        transform: Transform {
                            rotation: rot_y,
                            translation: vec3(0., 1., 0.),
                            scale: Vec3::splat(1.),
                        },
                        ..default()
                    },
                ));

                // Z
                cmd.spawn((
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                    TranslationWidget::Axis(Vec3::Z),
                    MaterialMeshBundle {
                        mesh: meshes.translate_line.clone(),
                        material: mats.translate_z.clone(),
                        transform: Transform {
                            rotation: rot_z,
                            translation: vec3(0., 0., 0.6),
                            scale: Vec3::splat(1.),
                        },
                        ..default()
                    },
                ));
                cmd.spawn((
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                    TranslationWidget::Axis(Vec3::Z),
                    MaterialMeshBundle {
                        mesh: meshes.translate_arrow.clone(),
                        material: mats.translate_z.clone(),
                        transform: Transform {
                            rotation: rot_z,
                            translation: vec3(0., 0., 1.),
                            scale: Vec3::splat(1.),
                        },
                        ..default()
                    },
                ));

                // YZ Plane
                cmd.spawn((
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                    TranslationWidget::Plane(Vec3::X),
                    MaterialMeshBundle {
                        mesh: meshes.translate_plane.clone(),
                        material: mats.translate_yz.clone(),
                        transform: Transform {
                            rotation: rot_x,
                            translation: vec3(0., 0.707, 0.707),
                            scale: Vec3::splat(1.),
                        },
                        ..default()
                    },
                ));
                // XZ Plane
                cmd.spawn((
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                    TranslationWidget::Plane(Vec3::Y),
                    MaterialMeshBundle {
                        mesh: meshes.translate_plane.clone(),
                        material: mats.translate_xz.clone(),
                        transform: Transform {
                            rotation: rot_y,
                            translation: vec3(0.707, 0., 0.707),
                            scale: Vec3::splat(1.),
                        },
                        ..default()
                    },
                ));
                // XY Plane
                cmd.spawn((
                    NotShadowCaster,
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                    TranslationWidget::Plane(Vec3::Z),
                    MaterialMeshBundle {
                        mesh: meshes.translate_plane.clone(),
                        material: mats.translate_xy.clone(),
                        transform: Transform {
                            rotation: rot_z,
                            translation: vec3(0.707, 0.707, 0.),
                            scale: Vec3::splat(1.),
                        },
                        ..default()
                    },
                ));
            });
        });
}

fn on_translate(
    event: Listener<Pointer<Drag>>,
    mut q_root: Query<(&mut Transform, &InitialTransform), With<TransformGizmoRoot>>,
    q_gizmo: Query<&GlobalTransform, (With<TransformGizmo>, Without<MainCamera>)>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    q_xlation_widget: Query<&TranslationWidget>,
) {
    let Ok((mut xform, init_xform)) = q_root.get_single_mut() else {
        return;
    };
    let Ok(gizmo_xform) = q_gizmo.get_single() else {
        return;
    };
    let Ok((cam, cam_xform)) = q_camera.get_single() else {
        return;
    };

    // Find the initial origin point in screen-space and add the total screen-space drag distance
    let origin_ws = init_xform.translation();
    let origin_ss = cam.world_to_viewport(cam_xform, origin_ws).unwrap();
    let target_ss = origin_ss + event.distance;

    // Create a camera-to-world ray that passes through the screen-space target point
    let ray = cam.viewport_to_world(cam_xform, target_ss).unwrap();

    use TranslationWidget::*;

    match q_xlation_widget.get(event.target) {
        Ok(Axis(axis)) => {
            // Transform the translation axis into world-space
            let axis_ws = gizmo_xform.affine().transform_vector3(*axis).normalize();

            // Find the plane vector that's orthogonal to the translation axis
            // and more perpendicular to the view direction
            let axis_tan = axis_ws.cross(cam_xform.back());
            let plane_vector = axis_ws.cross(axis_tan);

            // Find the intersection of the camera-to-world ray against that plane
            let hit_dist = ray.intersect_plane(origin_ws, plane_vector).unwrap();
            let hit_point = ray.origin + ray.direction * hit_dist;

            // Project the vector from origin -> hit point onto the translation
            // axis to constrain the translation
            let drag_vector = hit_point - origin_ws;
            let projection_length = drag_vector.dot(axis_ws);
            let target_ws = origin_ws + axis_ws * projection_length;

            xform.translation = target_ws;
        }
        Ok(plane) => {
            // Find the world-space normal of the plane we're translating on
            let plane_normal = match plane {
                ScreenSpace => cam_xform.back(),
                Plane(normal) => gizmo_xform.affine().transform_vector3(*normal),
                _ => unreachable!(),
            };

            // Find the intersection of the camera-to-world ray against that plane
            let hit_dist = ray.intersect_plane(origin_ws, plane_normal).unwrap();
            let target_ws = ray.origin + ray.direction * hit_dist;

            xform.translation = target_ws;
        }
        _ => {}
    }
}
