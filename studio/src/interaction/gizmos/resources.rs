use bevy::prelude::*;

use crate::{design_system, math::geo};

#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource)]
pub(super) struct GizmoMeshes {
    pub translate_line: Handle<Mesh>,
    pub translate_arrow: Handle<Mesh>,
    pub translate_plane: Handle<Mesh>,
    pub translate_ss: Handle<Mesh>,
}

#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource)]
pub(super) struct GizmoMats {
    pub translate_x: Handle<StandardMaterial>,
    pub translate_y: Handle<StandardMaterial>,
    pub translate_z: Handle<StandardMaterial>,
    pub translate_yz: Handle<StandardMaterial>,
    pub translate_xz: Handle<StandardMaterial>,
    pub translate_xy: Handle<StandardMaterial>,
    pub translate_ss: Handle<StandardMaterial>,
}

pub(super) fn init_resources(
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
    let translate_plane = ra_meshes.add(Mesh::from(shape::Plane {
        size: 0.25,
        subdivisions: 0,
    }));
    let translate_ss = ra_meshes.add(
        Mesh::try_from(shape::Icosphere {
            radius: 0.2,
            subdivisions: 3,
        })
        .unwrap(),
    );

    cmd.insert_resource(GizmoMeshes {
        translate_line,
        translate_arrow,
        translate_plane,
        translate_ss,
    });

    let translate_x = ra_mats.add(StandardMaterial {
        base_color: design_system::Color::RED,
        unlit: true,
        ..default()
    });
    let translate_yz = ra_mats.add(StandardMaterial {
        base_color: design_system::Color::RED,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    let translate_y = ra_mats.add(StandardMaterial {
        base_color: design_system::Color::GREEN,
        unlit: true,
        ..default()
    });
    let translate_xz = ra_mats.add(StandardMaterial {
        base_color: design_system::Color::GREEN,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    let translate_z = ra_mats.add(StandardMaterial {
        base_color: design_system::Color::BLUE,
        unlit: true,
        ..default()
    });
    let translate_xy = ra_mats.add(StandardMaterial {
        base_color: design_system::Color::BLUE,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    let translate_ss = ra_mats.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        ..default()
    });

    cmd.insert_resource(GizmoMats {
        translate_x,
        translate_y,
        translate_z,
        translate_yz,
        translate_xz,
        translate_xy,
        translate_ss,
    });
}
