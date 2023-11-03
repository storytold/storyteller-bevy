use bevy::{math::vec3, pbr::NotShadowCaster, prelude::*};
use bevy_mod_picking::prelude::{Listener, On, Out, Over, Pointer};

use crate::{design_system, math::geo};

use super::{Selection, TransformMode, TransformSpace};

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

pub struct GizmoPlugin;
impl Plugin for GizmoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(
            Update,
            despawn_existing_gizmo.run_if(resource_changed_or_removed::<Selection>()),
        );
        app.add_systems(
            Update,
            spawn_gizmo
                .after(despawn_existing_gizmo)
                .run_if(resource_exists_and_changed::<Selection>()),
        );
    }
}

fn setup(
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

impl TransformGizmo {
    pub fn compute_basis(owner_xform: &GlobalTransform, space: TransformSpace) -> Transform {
        let (inv_scale, inv_rotation, _) = owner_xform
            .affine()
            .inverse()
            .to_scale_rotation_translation();

        match space {
            TransformSpace::Local => Transform::from_scale(inv_scale),
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

    cmd.entity(ent)
        .insert(TransformGizmoRoot)
        .with_children(|cmd| {
            cmd.spawn(TransformGizmoBundle::new(xform, r_xform_mode.space))
                .with_children(|cmd| {
                    let rot_x = Quat::from_mat3(&Mat3::from_cols(Vec3::NEG_Y, Vec3::X, Vec3::Z));
                    let rot_y = Quat::IDENTITY;
                    let rot_z = Quat::from_mat3(&Mat3::from_cols(Vec3::X, Vec3::Z, Vec3::NEG_Y));

                    // Center sphere
                    cmd.spawn((
                        NotShadowCaster,
                        MaterialMeshBundle {
                            mesh: r_meshes.translate_center.clone(),
                            material: r_mats.translate_center.clone(),
                            transform: Transform::IDENTITY,
                            ..default()
                        },
                        On::<Pointer<Over>>::run(on_gizmo_element_hover),
                        On::<Pointer<Out>>::run(on_gizmo_element_unhover),
                    ));
                    // X
                    cmd.spawn((
                        NotShadowCaster,
                        MaterialMeshBundle {
                            mesh: r_meshes.translate_line.clone(),
                            material: r_mats.translate_x.clone(),
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
                            mesh: r_meshes.translate_arrow.clone(),
                            material: r_mats.translate_x.clone(),
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
                            mesh: r_meshes.translate_line.clone(),
                            material: r_mats.translate_y.clone(),
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
                            mesh: r_meshes.translate_arrow.clone(),
                            material: r_mats.translate_y.clone(),
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
                            mesh: r_meshes.translate_line.clone(),
                            material: r_mats.translate_z.clone(),
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
                            mesh: r_meshes.translate_arrow.clone(),
                            material: r_mats.translate_z.clone(),
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

fn on_gizmo_element_hover(
    ev: Listener<Pointer<Over>>,
    mut ra_mats: ResMut<Assets<StandardMaterial>>,
    q_target: Query<&Handle<StandardMaterial>>,
) {
    let mat = ra_mats.get_mut(q_target.get(ev.target).unwrap()).unwrap();
    mat.base_color *= 3.;
}

fn on_gizmo_element_unhover(
    ev: Listener<Pointer<Out>>,
    mut ra_mats: ResMut<Assets<StandardMaterial>>,
    q_target: Query<&Handle<StandardMaterial>>,
) {
    let mat = ra_mats.get_mut(q_target.get(ev.target).unwrap()).unwrap();
    mat.base_color *= 1. / 3.;
}
