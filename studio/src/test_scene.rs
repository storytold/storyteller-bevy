use bevy::{
    pbr::NotShadowCaster,
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::render_resource::{AsBindGroup, ShaderRef},
};

use crate::orbit_cam::{CameraMotionSystem, OrbitCam};

pub struct TestScenePlugin;

impl Plugin for TestScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::hsl(0.0, 0.0, 0.25)))
            .add_plugins(MaterialPlugin::<ProceduralGridMaterial>::default())
            .add_systems(Startup, spawn_scene)
            .add_systems(PostUpdate, move_plane_with_camera.after(CameraMotionSystem));
    }
}

#[derive(Component)]
struct WorldGridPlane;

fn spawn_scene(
    mut cmd: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mat_proc_grid: ResMut<Assets<ProceduralGridMaterial>>,
) {
    // Test scene
    cmd.spawn(SceneBundle {
        scene: assets.load("gltf/sample-room.gltf#Scene0"),
        ..default()
    });

    // Main light
    // TODO: get Blender to export enough metadata to automate light placements
    cmd.spawn(PointLightBundle {
        point_light: PointLight {
            color: Color::hex("FFE5B4").unwrap(),
            intensity: 200.,
            radius: 0.1,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.35263, 2.4348, 0.19601),
        ..default()
    });

    // Monitor glow
    cmd.spawn(PointLightBundle {
        point_light: PointLight {
            color: Color::WHITE,
            intensity: 6.,
            radius: 0.25,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_xyz(0.42615, 1.0706, -2.0705),
        ..default()
    });

    // Plane
    let plane = meshes.add(Mesh::from(shape::Plane {
        size: 200.0,
        subdivisions: 0,
    }));
    let proc_grid = mat_proc_grid.add(ProceduralGridMaterial {});

    cmd.spawn((
        SpatialBundle::from_transform(Transform::IDENTITY),
        WorldGridPlane,
    ))
    .with_children(|parent| {
        parent.spawn((
            MaterialMeshBundle {
                mesh: plane.clone(),
                material: proc_grid.clone(),
                transform: Transform::IDENTITY,
                ..default()
            },
            NotShadowCaster,
        ));
        parent.spawn((
            MaterialMeshBundle {
                mesh: plane,
                material: proc_grid,
                transform: Transform::IDENTITY.looking_to(Vec3::Z, Vec3::NEG_Y),
                ..default()
            },
            NotShadowCaster,
        ));
    });
}

fn move_plane_with_camera(
    mut plane: Query<(&WorldGridPlane, &mut Transform)>,
    camera: Query<&OrbitCam, Changed<OrbitCam>>,
) {
    if let Ok(&OrbitCam { focus, .. }) = camera.get_single() {
        let (_, mut xform) = plane.single_mut();

        xform.translation.x = focus.x;
        xform.translation.z = focus.z;
    }
}

#[derive(AsBindGroup, Clone, TypePath, TypeUuid)]
#[uuid = "CE331A13-F247-4F5B-A75E-670B2BD5776A"]
struct ProceduralGridMaterial {}

impl Material for ProceduralGridMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/procedural_grid.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
