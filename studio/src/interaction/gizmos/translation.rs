use bevy::{math::vec3, pbr::NotShadowCaster, prelude::*, render::view::RenderLayers};
use bevy_mod_picking::prelude::{Drag, DragEnd, DragStart, Listener, On, Out, Over, Pointer};

use crate::{interaction::TransformSpace, MainCamera};

use super::{
    on_gizmo_element_hover, on_gizmo_element_unhover, GizmoMats, GizmoMeshes, InitialTransform,
    TransformGizmo, TransformGizmoBundle, GIZMO_RENDER_LAYER,
};

#[derive(Component)]
pub(super) enum TranslationWidget {
    Axis(Vec3),
    Plane(Vec3),
    ScreenSpace,
}

#[inline]
pub(super) fn spawn_translation_gizmo(
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
        On::<Pointer<DragStart>>::run(on_translation_start),
        On::<Pointer<Drag>>::run(on_translate),
        On::<Pointer<DragEnd>>::run(on_translation_end),
    ))
    .with_children(|cmd| {
        use super::{CIRC_45, ROT_X, ROT_Y, ROT_Z};

        // Center sphere
        cmd.spawn((
            NotShadowCaster,
            RenderLayers::layer(GIZMO_RENDER_LAYER),
            TranslationWidget::ScreenSpace,
            MaterialMeshBundle {
                mesh: meshes.translate_ss.clone(),
                material: mats.translate_ss.handle.clone(),
                transform: Transform::IDENTITY,
                ..default()
            },
            mats.translate_ss.clone(),
        ));

        // X
        cmd.spawn((
            NotShadowCaster,
            RenderLayers::layer(GIZMO_RENDER_LAYER),
            TranslationWidget::Axis(Vec3::X),
            MaterialMeshBundle {
                mesh: meshes.translate_line.clone(),
                material: mats.translate_x.handle.clone(),
                transform: Transform {
                    rotation: ROT_X,
                    translation: vec3(0.6, 0., 0.),
                    ..default()
                },
                ..default()
            },
            mats.translate_x.clone(),
        ));
        cmd.spawn((
            NotShadowCaster,
            RenderLayers::layer(GIZMO_RENDER_LAYER),
            TranslationWidget::Axis(Vec3::X),
            MaterialMeshBundle {
                mesh: meshes.translate_arrow.clone(),
                material: mats.translate_x.handle.clone(),
                transform: Transform {
                    rotation: ROT_X,
                    translation: vec3(1., 0., 0.),
                    ..default()
                },
                ..default()
            },
            mats.translate_x.clone(),
        ));

        // Y
        cmd.spawn((
            NotShadowCaster,
            RenderLayers::layer(GIZMO_RENDER_LAYER),
            TranslationWidget::Axis(Vec3::Y),
            MaterialMeshBundle {
                mesh: meshes.translate_line.clone(),
                material: mats.translate_y.handle.clone(),
                transform: Transform {
                    rotation: ROT_Y,
                    translation: vec3(0., 0.6, 0.),
                    ..default()
                },
                ..default()
            },
            mats.translate_y.clone(),
        ));
        cmd.spawn((
            NotShadowCaster,
            RenderLayers::layer(GIZMO_RENDER_LAYER),
            TranslationWidget::Axis(Vec3::Y),
            MaterialMeshBundle {
                mesh: meshes.translate_arrow.clone(),
                material: mats.translate_y.handle.clone(),
                transform: Transform {
                    rotation: ROT_Y,
                    translation: vec3(0., 1., 0.),
                    ..default()
                },
                ..default()
            },
            mats.translate_y.clone(),
        ));

        // Z
        cmd.spawn((
            NotShadowCaster,
            RenderLayers::layer(GIZMO_RENDER_LAYER),
            TranslationWidget::Axis(Vec3::Z),
            MaterialMeshBundle {
                mesh: meshes.translate_line.clone(),
                material: mats.translate_z.handle.clone(),
                transform: Transform {
                    rotation: ROT_Z,
                    translation: vec3(0., 0., 0.6),
                    ..default()
                },
                ..default()
            },
            mats.translate_z.clone(),
        ));
        cmd.spawn((
            NotShadowCaster,
            RenderLayers::layer(GIZMO_RENDER_LAYER),
            TranslationWidget::Axis(Vec3::Z),
            MaterialMeshBundle {
                mesh: meshes.translate_arrow.clone(),
                material: mats.translate_z.handle.clone(),
                transform: Transform {
                    rotation: ROT_Z,
                    translation: vec3(0., 0., 1.),
                    ..default()
                },
                ..default()
            },
            mats.translate_z.clone(),
        ));

        // YZ Plane
        cmd.spawn((
            NotShadowCaster,
            RenderLayers::layer(GIZMO_RENDER_LAYER),
            TranslationWidget::Plane(Vec3::X),
            MaterialMeshBundle {
                mesh: meshes.translate_plane.clone(),
                material: mats.translate_yz.handle.clone(),
                transform: Transform {
                    rotation: ROT_X,
                    translation: vec3(0., CIRC_45, CIRC_45),
                    ..default()
                },
                ..default()
            },
            mats.translate_yz.clone(),
        ));
        // XZ Plane
        cmd.spawn((
            NotShadowCaster,
            RenderLayers::layer(GIZMO_RENDER_LAYER),
            TranslationWidget::Plane(Vec3::Y),
            MaterialMeshBundle {
                mesh: meshes.translate_plane.clone(),
                material: mats.translate_xz.handle.clone(),
                transform: Transform {
                    rotation: ROT_Y,
                    translation: vec3(CIRC_45, 0., CIRC_45),
                    ..default()
                },
                ..default()
            },
            mats.translate_xz.clone(),
        ));
        // XY Plane
        cmd.spawn((
            NotShadowCaster,
            RenderLayers::layer(GIZMO_RENDER_LAYER),
            TranslationWidget::Plane(Vec3::Z),
            MaterialMeshBundle {
                mesh: meshes.translate_plane.clone(),
                material: mats.translate_xy.handle.clone(),
                transform: Transform {
                    rotation: ROT_Z,
                    translation: vec3(CIRC_45, CIRC_45, 0.),
                    ..default()
                },
                ..default()
            },
            mats.translate_xy.clone(),
        ));
    });
}

pub(super) fn on_translation_start(
    _: Listener<Pointer<DragStart>>,
    mut cmd: Commands,
    q_xform: Query<(Entity, &GlobalTransform)>,
    q_gizmos: Query<&TransformGizmo>,
) {
    for gizmo_data in q_gizmos.iter() {
        if let Ok((ent, &world_xform)) = q_xform.get(gizmo_data.target) {
            cmd.entity(ent).insert(InitialTransform(world_xform));
        }
    }
}

pub(super) fn on_translation_end(
    _: Listener<Pointer<DragEnd>>,
    mut cmd: Commands,
    q_initial_transforms: Query<Entity, With<InitialTransform>>,
) {
    for ent in q_initial_transforms.iter() {
        cmd.entity(ent).remove::<InitialTransform>();
    }
}

pub(super) fn on_translate(
    event: Listener<Pointer<Drag>>,
    mut q_root: Query<(&mut Transform, &InitialTransform, Option<&Parent>)>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    q_gizmo: Query<(&GlobalTransform, &TransformGizmo), Without<MainCamera>>,
    q_widget: Query<&TranslationWidget>,
    q_all_global_transforms: Query<&GlobalTransform>,
) {
    let (gizmo_xform, gizmo_data) = q_gizmo.single();
    let Ok((mut target_transform, target_global_transform, target_parent)) =
        q_root.get_mut(gizmo_data.target)
    else {
        error!("Translation gizmo target not found");
        return;
    };
    let (cam, cam_xform) = q_camera.single();

    // Find the initial origin point in screen-space and add the total screen-space drag distance
    let origin_ws = target_global_transform.translation();
    let origin_ss = cam.world_to_viewport(cam_xform, origin_ws).unwrap();
    let target_ss = origin_ss + event.distance;

    // Create a camera-to-world ray that passes through the screen-space target point
    let ray = cam.viewport_to_world(cam_xform, target_ss).unwrap();

    use TranslationWidget::*;

    match q_widget.get(event.target) {
        Ok(Axis(axis)) => {
            // Transform the translation axis into world-space
            let axis_ws = gizmo_xform.affine().transform_vector3(*axis).normalize();

            // Find the plane vector that's orthogonal to the translation axis
            // and closest to parallel with the view direction
            let axis_tan = axis_ws.cross(cam_xform.back());
            let plane_vector = axis_ws.cross(axis_tan);

            // Find the intersection of the camera-to-world ray against that plane
            let Some(hit_dist) = ray.intersect_plane(origin_ws, Plane3d::new(plane_vector)) else {
                return;
            };
            let hit_point = ray.origin + ray.direction * hit_dist;

            // Project the vector from origin -> hit point onto the translation
            // axis to constrain the translation
            let drag_vector = hit_point - origin_ws;
            let projection_length = drag_vector.dot(axis_ws);
            let target_ws = origin_ws + axis_ws * projection_length;

            if let Some(target_parent) = target_parent {
                if let Ok(parent_global_transform) =
                    q_all_global_transforms.get(target_parent.get())
                {
                    let real_local_transform =
                        GlobalTransform::from(Transform::from_translation(target_ws))
                            .reparented_to(parent_global_transform);
                    target_transform.translation = real_local_transform.translation;
                }
            } else {
                target_transform.translation = target_ws;
            }
        }
        Ok(plane) => {
            // Find the world-space normal of the plane we're translating on
            let plane_normal = match plane {
                ScreenSpace => cam_xform.back(),
                Plane(normal) => gizmo_xform.affine().transform_vector3(*normal),
                _ => unreachable!(),
            };

            // Find the intersection of the camera-to-world ray against that plane
            let Some(hit_dist) = ray.intersect_plane(origin_ws, Plane3d::new(plane_normal)) else {
                return;
            };
            let target_ws = ray.origin + ray.direction * hit_dist;

            if let Some(target_parent) = target_parent {
                if let Ok(parent_global_transform) =
                    q_all_global_transforms.get(target_parent.get())
                {
                    let real_local_transform =
                        GlobalTransform::from(Transform::from_translation(target_ws))
                            .reparented_to(parent_global_transform);
                    target_transform.translation = real_local_transform.translation;
                }
            } else {
                target_transform.translation = target_ws;
            }
        }
        _ => {}
    }
}
