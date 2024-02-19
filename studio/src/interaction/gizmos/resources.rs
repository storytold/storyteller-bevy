use bevy::prelude::*;

use crate::{design_system, math::geo};

use super::material::GizmoMaterial;

#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource)]
pub(super) struct GizmoMeshes {
    pub translate_line: Handle<Mesh>,
    pub translate_arrow: Handle<Mesh>,
    pub translate_plane: Handle<Mesh>,
    pub translate_ss: Handle<Mesh>,
    pub rotate_ring: Handle<Mesh>,
    pub rotate_ball: Handle<Mesh>,
    pub rotate_ss: Handle<Mesh>,
}

#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource)]
pub(super) struct GizmoMats {
    pub translate_x: Handle<GizmoMaterial>,
    pub translate_y: Handle<GizmoMaterial>,
    pub translate_z: Handle<GizmoMaterial>,
    pub translate_yz: Handle<GizmoMaterial>,
    pub translate_xz: Handle<GizmoMaterial>,
    pub translate_xy: Handle<GizmoMaterial>,
    pub translate_ss: Handle<GizmoMaterial>,
    pub rotate_x: Handle<GizmoMaterial>,
    pub rotate_y: Handle<GizmoMaterial>,
    pub rotate_z: Handle<GizmoMaterial>,
    pub rotate_trackball: Handle<GizmoMaterial>,
    pub rotate_ss: Handle<GizmoMaterial>,
}

pub(super) fn init_resources(
    mut cmd: Commands,
    mut ra_meshes: ResMut<Assets<Mesh>>,
    mut ra_mats: ResMut<Assets<GizmoMaterial>>,
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
    let translate_plane = ra_meshes.add(Mesh::from(geo::Plane2Sided { size: 0.25 }));
    let translate_ss = ra_meshes.add(
        Mesh::try_from(shape::Icosphere {
            radius: 0.2,
            subdivisions: 3,
        })
        .unwrap(),
    );

    let rotate_ring = ra_meshes.add(Mesh::from(shape::Torus {
        radius: 1.,
        ring_radius: 0.04,
        subdivisions_segments: 40,
        subdivisions_sides: 6,
    }));
    let rotate_ball = ra_meshes.add(
        Mesh::try_from(shape::Icosphere {
            radius: 0.96,
            subdivisions: 4,
        })
        .unwrap(),
    );
    let rotate_ss = ra_meshes.add(Mesh::from(shape::Torus {
        radius: 1.2,
        ring_radius: 0.03,
        subdivisions_segments: 40,
        subdivisions_sides: 6,
    }));

    cmd.insert_resource(GizmoMeshes {
        translate_line,
        translate_arrow,
        translate_plane,
        translate_ss,
        rotate_ring,
        rotate_ball,
        rotate_ss,
    });

    let translate_x = ra_mats.add(design_system::Color::RED.into());
    let translate_yz = ra_mats.add(design_system::Color::RED.into());

    let translate_y = ra_mats.add(design_system::Color::GREEN.into());
    let translate_xz = ra_mats.add(design_system::Color::GREEN.into());

    let translate_z = ra_mats.add(design_system::Color::BLUE.into());
    let translate_xy = ra_mats.add(design_system::Color::BLUE.into());

    let translate_ss = ra_mats.add(Color::WHITE.into());

    let rotate_x = ra_mats.add(design_system::Color::RED.into());
    let rotate_y = ra_mats.add(design_system::Color::GREEN.into());
    let rotate_z = ra_mats.add(design_system::Color::BLUE.into());
    let rotate_trackball = ra_mats.add(Color::WHITE.with_a(0.0).into());
    let rotate_ss = ra_mats.add(Color::WHITE.into());

    cmd.insert_resource(GizmoMats {
        translate_x,
        translate_y,
        translate_z,
        translate_yz,
        translate_xz,
        translate_xy,
        translate_ss,
        rotate_x,
        rotate_y,
        rotate_z,
        rotate_trackball,
        rotate_ss,
    });
}
