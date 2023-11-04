use bevy::{ecs::system::EntityCommands, math::vec3, pbr::NotShadowCaster, prelude::*};
use bevy_mod_picking::prelude::{
    Drag, DragEnd, DragStart, Listener, ListenerInput, On, Out, Over, Pointer,
};

use crate::{camera_controller::CameraController, interaction::TransformSpace};

use super::{
    on_gizmo_element_hover, on_gizmo_element_unhover, GizmoMats, GizmoMeshes, TransformGizmoBundle,
    TransformGizmoRoot,
};

#[derive(Component, Deref, DerefMut)]
struct InitialTransform(GlobalTransform);

#[inline]
pub(super) fn spawn_translation_gizmo(
    cmd: &mut Commands,
    ent: Entity,
    xform: &GlobalTransform,
    space: TransformSpace,
    meshes: &GizmoMeshes,
    mats: &GizmoMats,
) {
    cmd.entity(ent)
        .insert(TransformGizmoRoot)
        .with_children(|cmd| {
            cmd.spawn(TransformGizmoBundle::new(xform, space))
                .with_children(|cmd| {
                    let rot_x = Quat::from_mat3(&Mat3::from_cols(Vec3::NEG_Y, Vec3::X, Vec3::Z));
                    let rot_y = Quat::IDENTITY;
                    let rot_z = Quat::from_mat3(&Mat3::from_cols(Vec3::X, Vec3::Z, Vec3::NEG_Y));

                    // Center sphere
                    cmd.spawn((
                        NotShadowCaster,
                        MaterialMeshBundle {
                            mesh: meshes.translate_center.clone(),
                            material: mats.translate_center.clone(),
                            transform: Transform::IDENTITY,
                            ..default()
                        },
                        On::<Pointer<Over>>::run(on_gizmo_element_hover),
                        On::<Pointer<Out>>::run(on_gizmo_element_unhover),
                        On::<Pointer<DragStart>>::run(on_translate_start),
                        On::<Pointer<Drag>>::run(on_translate_screenspace),
                        On::<Pointer<DragEnd>>::target_commands_mut(on_translate_end),
                    ));

                    // X
                    cmd.spawn((
                        NotShadowCaster,
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
                    ));
                    cmd.spawn((
                        NotShadowCaster,
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
                    ));

                    // Y
                    cmd.spawn((
                        NotShadowCaster,
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
                    ));
                    cmd.spawn((
                        NotShadowCaster,
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
                    ));

                    // Z
                    cmd.spawn((
                        NotShadowCaster,
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
                    ));
                    cmd.spawn((
                        NotShadowCaster,
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
                    ));
                });
        });
}

fn on_translate_start(
    _: Listener<Pointer<DragStart>>,
    mut cmd: Commands,
    q_xform: Query<(Entity, &GlobalTransform), With<TransformGizmoRoot>>,
) {
    if let Ok((ent, &xform)) = q_xform.get_single() {
        cmd.entity(ent).insert(InitialTransform(xform));
    };
}

fn on_translate_end(_: &ListenerInput<Pointer<DragEnd>>, cmd: &mut EntityCommands<'_, '_, '_>) {
    cmd.remove::<InitialTransform>();
}

fn on_translate_screenspace(
    event: Listener<Pointer<Drag>>,
    mut q_xform: Query<(&mut Transform, &InitialTransform), With<TransformGizmoRoot>>,
    q_cam_xform: Query<(&Camera, &GlobalTransform), With<CameraController>>,
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
