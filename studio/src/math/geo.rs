use bevy::{
    math::vec3,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

pub struct Cone {
    pub radius: f32,
    pub length: f32,
    pub resolution: u32,
}

impl From<Cone> for Mesh {
    fn from(c: Cone) -> Self {
        use std::f32::consts::PI;

        let num_verts = c.resolution + 2;
        let num_tris = c.resolution * 2;
        let num_indices = num_tris * 3;

        let mut positions = Vec::with_capacity(num_verts as usize);
        let mut indices = Vec::with_capacity(num_indices as usize);

        let rotator = Mat3::from_axis_angle(Vec3::Y, PI * 2. / c.resolution as f32);

        positions.push(Vec3::ZERO);

        let v_base = 0;
        let v_tip = num_verts - 1;

        let mut vert = vec3(0., 0., c.radius);
        for i in 0..c.resolution {
            positions.push(vert);
            vert = rotator * vert;

            let v1 = i + 1;
            let v2 = ((i + 2) % (c.resolution + 1)).max(1);

            #[rustfmt::skip]
            indices.extend([
                v2, v1, v_base,
                v1, v2, v_tip,
            ]);
        }

        positions.push(vec3(0., c.length, 0.));

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

        mesh
    }
}

pub struct Plane2Sided {
    pub size: f32,
}

impl Default for Plane2Sided {
    fn default() -> Self {
        Self { size: 1. }
    }
}

impl From<Plane2Sided> for Mesh {
    fn from(p: Plane2Sided) -> Self {
        let offset = p.size / 2.;

        // [x][ ][x]
        // [ ][ ][ ]
        // [x][ ][x]
        #[rustfmt::skip]
        let positions = vec![
            // 0:
            // [x][ ][ ]
            // [ ][ ][ ]
            // [ ][ ][ ]
            vec3(-offset, 0., -offset),
            // 1:
            // [ ][ ][x]
            // [ ][ ][ ]
            // [ ][ ][ ]
            vec3( offset, 0., -offset),
            // 2:
            // [ ][ ][ ]
            // [ ][ ][ ]
            // [ ][ ][x]
            vec3( offset, 0.,  offset),
            // 3:
            // [ ][ ][ ]
            // [ ][ ][ ]
            // [x][ ][ ]
            vec3(-offset, 0.,  offset),
        ];

        #[rustfmt::skip]
        let indices = vec![
            // Top side
            0, 3, 1,
            1, 3, 2,
            // Bottom side
            1, 3, 0,
            2, 3, 1,
        ];

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

        mesh
    }
}
