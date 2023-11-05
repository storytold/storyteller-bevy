use bevy::{math::vec3, pbr::NotShadowCaster, prelude::*, render::view::RenderLayers};
use bevy_mod_picking::prelude::{Drag, DragEnd, DragStart, Listener, On, Out, Over, Pointer};

use crate::{interaction::TransformSpace, MainCamera};

use super::{
    on_drag_end, on_drag_start, on_gizmo_element_hover, on_gizmo_element_unhover, GizmoMats,
    GizmoMeshes, InitialTransform, TransformGizmoBundle, TransformGizmoRoot, GIZMO_RENDER_LAYER,
};

#[derive(Component, Deref)]
struct TranslationAxis(Vec3);

#[derive(Component, Deref)]
struct TranslationPlane(Vec3);

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
            cmd.spawn(TransformGizmoBundle::new(xform, space, normalizing_scale))
                .with_children(|cmd| {
                    let rot_x = Quat::from_mat3(&Mat3::from_cols(Vec3::NEG_Y, Vec3::X, Vec3::Z));
                    let rot_y = Quat::IDENTITY;
                    let rot_z = Quat::from_mat3(&Mat3::from_cols(Vec3::X, Vec3::Z, Vec3::NEG_Y));

                    // Center sphere
                    cmd.spawn((
                        NotShadowCaster,
                        RenderLayers::layer(GIZMO_RENDER_LAYER),
                        MaterialMeshBundle {
                            mesh: meshes.translate_center.clone(),
                            material: mats.translate_center.clone(),
                            transform: Transform::IDENTITY,
                            ..default()
                        },
                        On::<Pointer<Over>>::run(on_gizmo_element_hover),
                        On::<Pointer<Out>>::run(on_gizmo_element_unhover),
                        On::<Pointer<DragStart>>::run(on_drag_start),
                        On::<Pointer<Drag>>::run(on_translate_screenspace),
                        On::<Pointer<DragEnd>>::target_commands_mut(on_drag_end),
                    ));

                    // X
                    cmd.spawn((
                        NotShadowCaster,
                        RenderLayers::layer(GIZMO_RENDER_LAYER),
                        TranslationAxis(Vec3::X),
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
                        On::<Pointer<Over>>::run(on_gizmo_element_hover),
                        On::<Pointer<Out>>::run(on_gizmo_element_unhover),
                        On::<Pointer<DragStart>>::run(on_drag_start),
                        On::<Pointer<Drag>>::run(on_translate),
                        On::<Pointer<DragEnd>>::target_commands_mut(on_drag_end),
                    ));
                    cmd.spawn((
                        NotShadowCaster,
                        RenderLayers::layer(GIZMO_RENDER_LAYER),
                        TranslationAxis(Vec3::X),
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
                        On::<Pointer<Over>>::run(on_gizmo_element_hover),
                        On::<Pointer<Out>>::run(on_gizmo_element_unhover),
                        On::<Pointer<DragStart>>::run(on_drag_start),
                        On::<Pointer<Drag>>::run(on_translate),
                        On::<Pointer<DragEnd>>::target_commands_mut(on_drag_end),
                    ));

                    // Y
                    cmd.spawn((
                        NotShadowCaster,
                        RenderLayers::layer(GIZMO_RENDER_LAYER),
                        TranslationAxis(Vec3::Y),
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
                        On::<Pointer<Over>>::run(on_gizmo_element_hover),
                        On::<Pointer<Out>>::run(on_gizmo_element_unhover),
                        On::<Pointer<DragStart>>::run(on_drag_start),
                        On::<Pointer<Drag>>::run(on_translate),
                        On::<Pointer<DragEnd>>::target_commands_mut(on_drag_end),
                    ));
                    cmd.spawn((
                        NotShadowCaster,
                        RenderLayers::layer(GIZMO_RENDER_LAYER),
                        TranslationAxis(Vec3::Y),
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
                        On::<Pointer<Over>>::run(on_gizmo_element_hover),
                        On::<Pointer<Out>>::run(on_gizmo_element_unhover),
                        On::<Pointer<DragStart>>::run(on_drag_start),
                        On::<Pointer<Drag>>::run(on_translate),
                        On::<Pointer<DragEnd>>::target_commands_mut(on_drag_end),
                    ));

                    // Z
                    cmd.spawn((
                        NotShadowCaster,
                        RenderLayers::layer(GIZMO_RENDER_LAYER),
                        TranslationAxis(Vec3::Z),
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
                        On::<Pointer<Over>>::run(on_gizmo_element_hover),
                        On::<Pointer<Out>>::run(on_gizmo_element_unhover),
                        On::<Pointer<DragStart>>::run(on_drag_start),
                        On::<Pointer<Drag>>::run(on_translate),
                        On::<Pointer<DragEnd>>::target_commands_mut(on_drag_end),
                    ));
                    cmd.spawn((
                        NotShadowCaster,
                        RenderLayers::layer(GIZMO_RENDER_LAYER),
                        TranslationAxis(Vec3::Z),
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
                        On::<Pointer<Over>>::run(on_gizmo_element_hover),
                        On::<Pointer<Out>>::run(on_gizmo_element_unhover),
                        On::<Pointer<DragStart>>::run(on_drag_start),
                        On::<Pointer<Drag>>::run(on_translate),
                        On::<Pointer<DragEnd>>::target_commands_mut(on_drag_end),
                    ));
                });
        });
}

fn on_translate_screenspace(
    event: Listener<Pointer<Drag>>,
    mut q_xform: Query<(&mut Transform, &InitialTransform), With<TransformGizmoRoot>>,
    q_cam_xform: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let Ok((mut xform, init_xform)) = q_xform.get_single_mut() else {
        return;
    };
    let Ok((cam, cam_xform)) = q_cam_xform.get_single() else {
        return;
    };

    let origin_ws = init_xform.translation();
    let origin_ss = cam.world_to_viewport(cam_xform, origin_ws).unwrap();

    let target_ss = origin_ss + event.distance;
    let ray = cam.viewport_to_world(cam_xform, target_ss).unwrap();
    let hit_dist = ray.intersect_plane(origin_ws, cam_xform.back()).unwrap();
    let target_ws = ray.origin + ray.direction * hit_dist;

    xform.translation = target_ws;
}

fn on_translate(
    event: Listener<Pointer<Drag>>,
    mut q_xform: Query<
        (&mut Transform, &GlobalTransform, &InitialTransform),
        (With<TransformGizmoRoot>, Without<MainCamera>),
    >,
    q_cam_xform: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    q_xlation_axis: Query<&TranslationAxis>,
    _q_xlation_plane: Query<&TranslationPlane>, // TODO
) {
    let Ok((mut xform, world_xform, init_xform)) = q_xform.get_single_mut() else {
        return;
    };
    let Ok((cam, cam_xform)) = q_cam_xform.get_single() else {
        return;
    };

    let origin_ws = init_xform.translation();
    let origin_ss = cam.world_to_viewport(cam_xform, origin_ws).unwrap();
    let target_ss = origin_ss + event.distance;

    if let Ok(axis_ws) = q_xlation_axis
        .get(event.target)
        .map(|axis| world_xform.affine().transform_vector3(**axis).normalize())
    {
        let (axis_tan, axis_binorm) = axis_ws.any_orthonormal_pair();
        let ray = cam.viewport_to_world(cam_xform, target_ss).unwrap();
        let Some(hit_dist) = ray
            .intersect_plane(origin_ws, axis_tan)
            .or_else(|| ray.intersect_plane(origin_ws, axis_binorm))
        else {
            return;
        };

        let hit_point = ray.origin + ray.direction * hit_dist;
        let drag_vector = hit_point - origin_ws;
        let projection_length = drag_vector.dot(axis_ws);
        let target_ws = origin_ws + axis_ws * projection_length;

        xform.translation = target_ws;
    }
}
