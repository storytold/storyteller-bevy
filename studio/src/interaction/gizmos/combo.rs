use bevy::{math::vec3, pbr::NotShadowCaster, prelude::*, render::view::RenderLayers};
use bevy_mod_picking::prelude::*;

use crate::interaction::TransformSpace;

use super::{
    on_gizmo_element_hover, on_gizmo_element_unhover,
    rotation::{on_rotate, on_rotation_end, on_rotation_start, RotationWidget},
    translation::{on_translate, on_translation_end, on_translation_start, TranslationWidget},
    GizmoMats, GizmoMeshes, TransformGizmoBundle, TransformGizmoRoot, GIZMO_RENDER_LAYER,
};

#[inline]
pub(super) fn spawn_combo_translation_rotation_gizmo(
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
                    use super::{ROT_X, ROT_Y, ROT_Z};

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
                        On::<Pointer<Over>>::run(on_gizmo_element_hover),
                        On::<Pointer<Out>>::run(on_gizmo_element_unhover),
                        On::<Pointer<DragStart>>::run(on_translation_start),
                        On::<Pointer<Drag>>::run(on_translate),
                        On::<Pointer<DragEnd>>::target_commands_mut(on_translation_end),
                    ));

                    // Rotate X
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
                        On::<Pointer<DragStart>>::run(on_rotation_start),
                        On::<Pointer<Drag>>::run(on_rotate),
                        On::<Pointer<DragEnd>>::run(on_rotation_end),
                    ));
                    // Translate X
                    cmd.spawn((
                        NotShadowCaster,
                        RenderLayers::layer(GIZMO_RENDER_LAYER),
                        TranslationWidget::Axis(Vec3::X),
                        MaterialMeshBundle {
                            mesh: meshes.translate_arrow.clone(),
                            material: mats.translate_x.clone(),
                            transform: Transform {
                                rotation: ROT_X,
                                translation: vec3(1.2, 0., 0.),
                                ..default()
                            },
                            ..default()
                        },
                        On::<Pointer<Over>>::run(on_gizmo_element_hover),
                        On::<Pointer<Out>>::run(on_gizmo_element_unhover),
                        On::<Pointer<DragStart>>::run(on_translation_start),
                        On::<Pointer<Drag>>::run(on_translate),
                        On::<Pointer<DragEnd>>::target_commands_mut(on_translation_end),
                    ));

                    // Rotate Y
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
                        On::<Pointer<DragStart>>::run(on_rotation_start),
                        On::<Pointer<Drag>>::run(on_rotate),
                        On::<Pointer<DragEnd>>::run(on_rotation_end),
                    ));
                    // Translate Y
                    cmd.spawn((
                        NotShadowCaster,
                        RenderLayers::layer(GIZMO_RENDER_LAYER),
                        TranslationWidget::Axis(Vec3::Y),
                        MaterialMeshBundle {
                            mesh: meshes.translate_arrow.clone(),
                            material: mats.translate_y.clone(),
                            transform: Transform {
                                rotation: ROT_Y,
                                translation: vec3(0., 1.2, 0.),
                                ..default()
                            },
                            ..default()
                        },
                        On::<Pointer<Over>>::run(on_gizmo_element_hover),
                        On::<Pointer<Out>>::run(on_gizmo_element_unhover),
                        On::<Pointer<DragStart>>::run(on_translation_start),
                        On::<Pointer<Drag>>::run(on_translate),
                        On::<Pointer<DragEnd>>::target_commands_mut(on_translation_end),
                    ));

                    // Rotate Z
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
                        On::<Pointer<DragStart>>::run(on_rotation_start),
                        On::<Pointer<Drag>>::run(on_rotate),
                        On::<Pointer<DragEnd>>::run(on_rotation_end),
                    ));
                    // Translate Z
                    cmd.spawn((
                        NotShadowCaster,
                        RenderLayers::layer(GIZMO_RENDER_LAYER),
                        TranslationWidget::Axis(Vec3::Z),
                        MaterialMeshBundle {
                            mesh: meshes.translate_arrow.clone(),
                            material: mats.translate_z.clone(),
                            transform: Transform {
                                rotation: ROT_Z,
                                translation: vec3(0., 0., 1.2),
                                ..default()
                            },
                            ..default()
                        },
                        On::<Pointer<Over>>::run(on_gizmo_element_hover),
                        On::<Pointer<Out>>::run(on_gizmo_element_unhover),
                        On::<Pointer<DragStart>>::run(on_translation_start),
                        On::<Pointer<Drag>>::run(on_translate),
                        On::<Pointer<DragEnd>>::target_commands_mut(on_translation_end),
                    ));
                });
        });
}
