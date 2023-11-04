use bevy::{
    core_pipeline::clear_color::ClearColorConfig, ecs::system::EntityCommands, math::vec3,
    prelude::*, render::view::RenderLayers,
};
use bevy_mod_picking::prelude::{DragEnd, DragStart, Listener, ListenerInput, Out, Over, Pointer};

use crate::{
    design_system,
    math::{geo, IsNonUniform},
    MainCamera,
};

use self::translation::spawn_translation_gizmo;

use super::{Selection, TransformMode, TransformSpace, TransformType};

mod translation;

#[derive(Component)]
pub struct TransformGizmoRoot;

#[derive(Component)]
pub struct TransformGizmo;

#[derive(Bundle)]
pub struct TransformGizmoBundle {
    name: Name,
    marker: TransformGizmo,
    spatial: SpatialBundle,
}

pub(super) const GIZMO_RENDER_LAYER: u8 = 1;

#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource)]
pub struct GizmoMeshes {
    pub translate_line: Handle<Mesh>,
    pub translate_arrow: Handle<Mesh>,
    pub translate_center: Handle<Mesh>,
}

#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource)]
pub struct GizmoMats {
    pub translate_x: Handle<StandardMaterial>,
    pub translate_y: Handle<StandardMaterial>,
    pub translate_z: Handle<StandardMaterial>,
    pub translate_center: Handle<StandardMaterial>,
}

#[derive(Resource, Deref, DerefMut)]
pub(super) struct GizmoCamera(Entity);

pub struct GizmoPlugin;
impl Plugin for GizmoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_resources);
        app.add_systems(
            PreUpdate,
            spawn_gizmo_camera.run_if(not(resource_exists::<GizmoCamera>())),
        );
        app.add_systems(
            Update,
            (
                despawn_existing_gizmo.run_if(
                    resource_changed_or_removed::<Selection>()
                        .or_else(resource_changed::<TransformMode>()),
                ),
                spawn_gizmo.after(despawn_existing_gizmo).run_if(
                    not(any_with_component::<TransformGizmoRoot>()).and_then(
                        resource_exists_and_changed::<Selection>().or_else(
                            resource_exists::<Selection>()
                                .and_then(resource_changed::<TransformMode>()),
                        ),
                    ),
                ),
            ),
        );
    }
}

fn init_resources(
    mut cmd: Commands,
    mut ra_meshes: ResMut<Assets<Mesh>>,
    mut ra_mats: ResMut<Assets<StandardMaterial>>,
) {
    let translate_line = ra_meshes.add(Mesh::from(shape::Cylinder {
        radius: 0.03,
        height: 0.8,
        resolution: 6,
        segments: 1,
    }));
    let translate_arrow = ra_meshes.add(Mesh::from(geo::Cone {
        radius: 0.115,
        length: 0.4,
        resolution: 12,
    }));
    let translate_center = ra_meshes.add(
        Mesh::try_from(shape::Icosphere {
            radius: 0.2,
            subdivisions: 3,
        })
        .unwrap(),
    );

    cmd.insert_resource(GizmoMeshes {
        translate_line,
        translate_arrow,
        translate_center,
    });

    let translate_x = ra_mats.add(StandardMaterial {
        base_color: design_system::Color::RED,
        unlit: true,
        ..default()
    });
    let translate_y = ra_mats.add(StandardMaterial {
        base_color: design_system::Color::GREEN,
        unlit: true,
        ..default()
    });
    let translate_z = ra_mats.add(StandardMaterial {
        base_color: design_system::Color::BLUE,
        unlit: true,
        ..default()
    });
    let translate_center = ra_mats.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        ..default()
    });

    cmd.insert_resource(GizmoMats {
        translate_x,
        translate_y,
        translate_z,
        translate_center,
    });
}

fn spawn_gizmo_camera(mut cmd: Commands, q_cam_controller: Query<Entity, With<MainCamera>>) {
    if let Ok(ent) = q_cam_controller.get_single() {
        let mut gizmo_cam = None;

        cmd.entity(ent).with_children(|cmd| {
            gizmo_cam = Some(
                cmd.spawn((
                    Name::new("Gizmo Camera"),
                    Camera3dBundle {
                        camera_3d: Camera3d {
                            clear_color: ClearColorConfig::None,
                            ..default()
                        },
                        camera: Camera {
                            hdr: true,
                            order: 1,
                            ..default()
                        },
                        ..default()
                    },
                    RenderLayers::layer(GIZMO_RENDER_LAYER),
                ))
                .id(),
            );
        });

        if let Some(ent) = gizmo_cam {
            cmd.insert_resource(GizmoCamera(ent));
        }
    }
}

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
    pub fn new(parent_xform: &GlobalTransform, space: TransformSpace) -> Self {
        let basis = TransformGizmo::compute_basis(parent_xform, space);

        Self {
            name: Name::new("TransformGizmo"),
            marker: TransformGizmo,
            spatial: SpatialBundle::from_transform(basis),
        }
    }
}

pub(super) fn despawn_existing_gizmo(
    mut cmd: Commands,
    q_gizmo_root: Query<(Entity, &Children), With<TransformGizmoRoot>>,
    q_gizmos: Query<Entity, With<TransformGizmo>>,
) {
    if let Ok((ent, children)) = q_gizmo_root.get_single() {
        if let Some(child) = children.iter().find_map(|ent| q_gizmos.get(*ent).ok()) {
            cmd.entity(child).despawn_recursive();
        }

        cmd.entity(ent).remove::<TransformGizmoRoot>();
    }
}

pub(super) fn spawn_gizmo(
    mut cmd: Commands,
    r_selection: Res<Selection>,
    r_xform_mode: Res<TransformMode>,
    r_meshes: Res<GizmoMeshes>,
    r_mats: Res<GizmoMats>,
    q_xform: Query<&GlobalTransform>,
) {
    let ent = **r_selection;
    let Ok(xform) = q_xform.get(ent) else {
        return;
    };

    match r_xform_mode.ttype {
        TransformType::TRANSLATE => {
            spawn_translation_gizmo(&mut cmd, ent, xform, r_xform_mode.space, &r_meshes, &r_mats);
        }
        _ => {
            // TODO
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub(super) struct InitialTransform(GlobalTransform);

pub(super) fn on_drag_start(
    _: Listener<Pointer<DragStart>>,
    mut cmd: Commands,
    q_xform: Query<(Entity, &GlobalTransform), With<TransformGizmoRoot>>,
) {
    if let Ok((ent, &xform)) = q_xform.get_single() {
        cmd.entity(ent).insert(InitialTransform(xform));
    };
}

pub(super) fn on_drag_end(
    _: &ListenerInput<Pointer<DragEnd>>,
    cmd: &mut EntityCommands<'_, '_, '_>,
) {
    cmd.remove::<InitialTransform>();
}

pub(super) fn on_gizmo_element_hover(
    ev: Listener<Pointer<Over>>,
    mut ra_mats: ResMut<Assets<StandardMaterial>>,
    q_target: Query<&Handle<StandardMaterial>>,
) {
    let mat = ra_mats.get_mut(q_target.get(ev.target).unwrap()).unwrap();
    mat.base_color *= 3.;
}

pub(super) fn on_gizmo_element_unhover(
    ev: Listener<Pointer<Out>>,
    mut ra_mats: ResMut<Assets<StandardMaterial>>,
    q_target: Query<&Handle<StandardMaterial>>,
) {
    let mat = ra_mats.get_mut(q_target.get(ev.target).unwrap()).unwrap();
    mat.base_color *= 1. / 3.;
}
