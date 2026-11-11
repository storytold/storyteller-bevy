use bevy::prelude::*;

use crate::{design_system, math::geo};

use super::material::*;

#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource)]
pub(super) struct GizmoMeshes {
    pub translate_line: Handle<Mesh>,
    pub translate_arrow: Handle<Mesh>,
    pub scale_cube: Handle<Mesh>,
    pub uniform_scale_cube: Handle<Mesh>,
    pub translate_plane: Handle<Mesh>,
    pub translate_ss: Handle<Mesh>,
    pub rotate_ring: Handle<Mesh>,
    pub rotate_ball: Handle<Mesh>,
    pub rotate_ss: Handle<Mesh>,
}

#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component)]
pub(super) struct GizmoMaterialSet {
    pub handle: Handle<GizmoMaterial>,
    base_color: Color,
    hover_color: Color,
}

impl GizmoMaterialSet {
    pub fn from_base_color(assets: &mut Assets<GizmoMaterial>, base_color: Color) -> Self {
        let handle = assets.add(GizmoMaterial::from(base_color));
        let hover_color = base_color * 2.;

        Self {
            handle,
            base_color,
            hover_color,
        }
    }

    pub fn set_default(&mut self, assets: &mut Assets<GizmoMaterial>) {
        if let Some(mat) = assets.get_mut(&self.handle) {
            mat.color = self.base_color;
        }
    }

    pub fn set_hover(&mut self, assets: &mut Assets<GizmoMaterial>) {
        if let Some(mat) = assets.get_mut(&self.handle) {
            mat.color = self.hover_color;
        }
    }
}

#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource)]
pub(super) struct GizmoMats {
    pub translate_x: GizmoMaterialSet,
    pub translate_y: GizmoMaterialSet,
    pub translate_z: GizmoMaterialSet,
    pub translate_yz: GizmoMaterialSet,
    pub translate_xz: GizmoMaterialSet,
    pub translate_xy: GizmoMaterialSet,
    pub translate_ss: GizmoMaterialSet,
    pub scale_x: GizmoMaterialSet,
    pub scale_y: GizmoMaterialSet,
    pub scale_z: GizmoMaterialSet,
    pub rotate_x: GizmoMaterialSet,
    pub rotate_y: GizmoMaterialSet,
    pub rotate_z: GizmoMaterialSet,
    pub rotate_trackball: GizmoMaterialSet,
    pub rotate_ss: GizmoMaterialSet,
}

#[allow(deprecated)]
pub(super) fn init_resources(
    mut cmd: Commands,
    mut ra_meshes: ResMut<Assets<Mesh>>,
    mut ra_mats: ResMut<Assets<GizmoMaterial>>,
) {
    let translate_line = ra_meshes.add(Cylinder {
        radius: 0.03,
        half_height: 0.4,
    });

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

    let scale_cube = ra_meshes.add(Mesh::from(shape::Cube { size: 0.2 }));
    let uniform_scale_cube = ra_meshes.add(Mesh::from(shape::Cube { size: 0.333 }));

    cmd.insert_resource(GizmoMeshes {
        translate_line,
        translate_arrow,
        translate_plane,
        translate_ss,
        rotate_ring,
        rotate_ball,
        rotate_ss,
        scale_cube,
        uniform_scale_cube,
    });

    let translate_x = GizmoMaterialSet::from_base_color(&mut ra_mats, design_system::Color::RED);
    let translate_yz = GizmoMaterialSet::from_base_color(&mut ra_mats, design_system::Color::RED);

    let translate_y = GizmoMaterialSet::from_base_color(&mut ra_mats, design_system::Color::GREEN);
    let translate_xz = GizmoMaterialSet::from_base_color(&mut ra_mats, design_system::Color::GREEN);

    let translate_z = GizmoMaterialSet::from_base_color(&mut ra_mats, design_system::Color::BLUE);
    let translate_xy = GizmoMaterialSet::from_base_color(&mut ra_mats, design_system::Color::BLUE);

    let translate_ss = GizmoMaterialSet::from_base_color(&mut ra_mats, Color::WHITE);

    let rotate_x = GizmoMaterialSet::from_base_color(&mut ra_mats, design_system::Color::RED);
    let rotate_y = GizmoMaterialSet::from_base_color(&mut ra_mats, design_system::Color::GREEN);
    let rotate_z = GizmoMaterialSet::from_base_color(&mut ra_mats, design_system::Color::BLUE);
    let rotate_ss = GizmoMaterialSet::from_base_color(&mut ra_mats, Color::WHITE);

    let scale_x = GizmoMaterialSet::from_base_color(&mut ra_mats, design_system::Color::RED);
    let scale_y = GizmoMaterialSet::from_base_color(&mut ra_mats, design_system::Color::GREEN);
    let scale_z = GizmoMaterialSet::from_base_color(&mut ra_mats, design_system::Color::BLUE);

    let rotate_trackball = GizmoMaterialSet {
        handle: ra_mats.add(GizmoMaterial {
            color: Color::WHITE.with_a(0.0),
            blend_mode: AlphaMode::Blend,
        }),
        base_color: Color::WHITE.with_a(0.0),
        hover_color: Color::WHITE.with_a(0.05),
    };

    cmd.insert_resource(GizmoMats {
        translate_x,
        translate_y,
        translate_z,
        translate_yz,
        translate_xz,
        translate_xy,
        translate_ss,
        scale_x,
        scale_y,
        scale_z,
        rotate_x,
        rotate_y,
        rotate_z,
        rotate_trackball,
        rotate_ss,
    });
}
