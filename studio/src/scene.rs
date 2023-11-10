use bevy::{
    asset::LoadState,
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings},
        fxaa::Fxaa,
        tonemapping::Tonemapping,
    },
    gltf::{self, Gltf},
    math::vec3,
    pbr::NotShadowCaster,
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::render_resource::{AsBindGroup, ShaderRef},
    utils::HashSet,
};
use bevy_mod_picking::PickableBundle;

use crate::{
    camera_controller::{CameraController, CameraControllerBundle, CameraMotionSystem},
    interaction::Selectable,
    skybox::SkyboxId,
    MainCamera,
};

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::hsl(0.0, 0.0, 0.25)))
            .insert_resource(Msaa::Off)
            .add_plugins(MaterialPlugin::<ProceduralGridMaterial>::default())
            .add_systems(Startup, initialize)
            .add_systems(
                Update,
                mark_scene_objects.run_if(
                    resource_exists::<NewGltf>()
                        .and_then(gltf_loaded)
                        .and_then(gltf_scene_spawned),
                ),
            )
            .add_systems(PostUpdate, move_plane_with_camera.after(CameraMotionSystem));

        #[cfg(feature = "wasm")]
        app.add_systems(PostUpdate, wasm::notify_entity_spawn_changes.pipe(error));
    }
}

#[derive(Component)]
struct WorldGridPlane;

#[derive(Resource, Deref, DerefMut)]
struct NewGltf(pub Handle<Gltf>);

#[derive(Component)]
struct Inspectable;

fn initialize(
    mut cmd: Commands,
    assets: Res<AssetServer>,
    mut ra_meshes: ResMut<Assets<Mesh>>,
    mut ra_mat_proc_grid: ResMut<Assets<ProceduralGridMaterial>>,
) {
    // Skybox
    cmd.insert_resource(SkyboxId("test_scene".into()));

    // glTF scene
    cmd.insert_resource(NewGltf(assets.load("gltf/sample-room.gltf")));
    cmd.spawn((
        Name::new("Scene"),
        Inspectable,
        SceneBundle {
            scene: assets.load("gltf/sample-room.gltf#Scene0"),
            ..default()
        },
    ));

    // Camera, graphics settings
    let focus = vec3(0.35011, 0.5, -0.99576);
    let camera_xform = Transform::from_xyz(-2.5, 4.5, 5.0).looking_at(focus, Vec3::Y);
    cmd.spawn((
        Name::new("Main Camera"),
        MainCamera,
        Camera3dBundle {
            transform: camera_xform,
            tonemapping: Tonemapping::BlenderFilmic,
            camera: Camera {
                hdr: true,
                ..default()
            },
            ..default()
        },
        CameraControllerBundle::new(focus, camera_xform.translation),
        BloomSettings {
            composite_mode: BloomCompositeMode::EnergyConserving,
            intensity: 0.25,
            prefilter_settings: BloomPrefilterSettings {
                threshold: 0.8,
                threshold_softness: 0.5,
            },
            ..default()
        },
        Fxaa::default(),
    ));

    // Main light
    // TODO: get Blender to export enough metadata to automate light placements
    cmd.spawn((
        Name::new("Ceiling Light"),
        Selectable,
        Inspectable,
        PointLightBundle {
            point_light: PointLight {
                color: Color::hex("FFE5B4").unwrap(),
                intensity: 200.,
                radius: 0.1,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(0.35263, 2.4348, 0.19601),
            ..default()
        },
    ));

    // Monitor glow
    cmd.spawn((
        Name::new("Monitor Glow"),
        Selectable,
        Inspectable,
        PointLightBundle {
            point_light: PointLight {
                color: Color::WHITE,
                intensity: 6.,
                radius: 0.25,
                shadows_enabled: false,
                ..default()
            },
            transform: Transform::from_xyz(0.42615, 1.0706, -2.0705),
            ..default()
        },
    ));

    // Plane
    let plane = ra_meshes.add(Mesh::from(shape::Plane {
        size: 200.0,
        subdivisions: 0,
    }));
    let proc_grid = ra_mat_proc_grid.add(ProceduralGridMaterial {});

    cmd.spawn((
        Name::new("World Grid"),
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

fn gltf_loaded(assets: Res<AssetServer>, gltf: Res<NewGltf>) -> bool {
    assets.get_load_state(&**gltf) == LoadState::Loaded
}

fn gltf_scene_spawned(
    r_assets: Res<AssetServer>,
    r_gltf: Res<NewGltf>,
    ra_gltfs: Res<Assets<Gltf>>,
    q_scenes: Query<&Handle<Scene>, (With<Name>, With<Children>)>,
) -> bool {
    let gltf = ra_gltfs.get(&r_gltf).unwrap();
    let Some(gltf_scene_path) = gltf
        .default_scene
        .as_ref()
        .and_then(|handle| r_assets.get_handle_path(handle))
    else {
        return false;
    };

    q_scenes.iter().any(|handle| {
        let Some(path) = r_assets.get_handle_path(handle) else {
            return false;
        };
        path == gltf_scene_path
    })
}

fn mark_scene_objects(
    mut cmd: Commands,
    r_gltf: Res<NewGltf>,
    r_assets: Res<AssetServer>,
    ra_gltfs: Res<Assets<Gltf>>,
    ra_gltf_nodes: Res<Assets<gltf::GltfNode>>,
    ra_gltf_meshes: Res<Assets<gltf::GltfMesh>>,
    q_parents: Query<(Entity, &Name, &Children), Without<Selectable>>,
    q_primitives: Query<&Handle<Mesh>>,
) {
    let gltf = ra_gltfs.get(&r_gltf).unwrap();

    for (ent_parent, name, children) in q_parents.iter() {
        if gltf.named_nodes.contains_key(&**name) {
            cmd.entity(ent_parent).insert(Inspectable);
        }

        // See if this name matches any of the meshes or nodes in our glTF scene
        let Some(gltf_mesh) = gltf
            .named_meshes
            .get(&**name)
            .cloned()
            .or_else(|| {
                gltf.named_nodes.get(&**name).and_then(|node_handle| {
                    ra_gltf_nodes
                        .get(node_handle)
                        .and_then(|node| node.mesh.clone())
                })
            })
            .and_then(|handle| ra_gltf_meshes.get(&handle))
        else {
            continue;
        };

        // FIXME: These hashsets don't actually contain anything. Ever. -_-
        //        This has "worked" this whole time because {} == {}

        // Collect all of the mesh handles in the glTF mesh
        let prims_from_gltf = gltf_mesh
            .primitives
            .iter()
            .filter_map(|prim| r_assets.get_handle_path(&prim.mesh))
            .collect::<HashSet<_>>();

        // Collect all of the mesh handles in this entity's children
        let prims_from_ecs = children
            .iter()
            .filter_map(|ent_child| {
                q_primitives
                    .get(*ent_child)
                    .ok()
                    .and_then(|handle| r_assets.get_handle_path(handle))
            })
            .collect::<HashSet<_>>();


        // If the sets match, this is the entity that represents the glTF mesh
        if prims_from_gltf == prims_from_ecs {
            cmd.entity(ent_parent).insert(Selectable);

            for &ent_child in children.iter() {
                if q_primitives.contains(ent_child) {
                    cmd.entity(ent_child).insert(PickableBundle::default());
                }
            }
        }
    }

    cmd.remove_resource::<NewGltf>();
}

fn move_plane_with_camera(
    mut plane: Query<(&WorldGridPlane, &mut Transform)>,
    camera: Query<&CameraController, Changed<CameraController>>,
) {
    if let Ok(&CameraController { focus, .. }) = camera.get_single() {
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

#[cfg(feature = "wasm")]
mod wasm {
    use std::collections::VecDeque;

    use bevy::{prelude::*, utils::HashSet, window::PrimaryWindow};
    use serde::Serialize;
    use wasm_bindgen::prelude::*;

    use crate::wasm::{self, AsJson};

    use super::Inspectable;

    #[wasm_bindgen(typescript_custom_section)]
    const TYPE_DECLARATIONS: &str = r#"
export interface SceneObject {
    /** The entity ID. */
    id: string;
    /** A user-friendly name for displaying in the UI */
    name: string;
    /** The IDs of this object's children in the hierarchy */
    children: string[];
    /** The ID of this object's parent in the hierarchy, if it has one */
    parent: string | null;
    /** The local-space transform */
    transform: Transform;
    /** The world-space transform */
    globalTransform: GlobalTransform;
}

/**
 * An event indicating that a new entity has spawned.
 */
export interface EntitySpawnEvent extends CustomEvent {
    type: "entity-spawn";
    detail: SceneObject;
}

/**
 * An event indicating that an entity has been de-spawned.
 */
export interface EntityDespawnEvent extends CustomEvent {
    type: "entity-despawn";
    /** The entity ID */
    detail: string;
}

/**
 * An event indicating that multiple entities have spawned.
 */
export interface EntityMultiSpawnEvent extends CustomEvent {
    type: "entity-multi-spawn";
    detail: SceneObject[];
}

declare global {
    export interface HTMLElementEventMap {
        "entity-spawn": EntitySpawnEvent;
        "entity-despawn": EntityDespawnEvent;
        "entity-multi-spawn": EntityMultiSpawnEvent;
    }
}
    "#;

    #[derive(Serialize)]
    pub(super) struct SceneObject {
        id: String,
        name: String,
        children: Vec<String>,
        parent: Option<String>,
        transform: serde_json::Value,
        #[serde(rename(serialize = "globalTransform"))]
        global_transform: serde_json::Value,
    }

    struct SceneObjectBuilder<'a> {
        id: Entity,
        name: Option<String>,
        children: Option<Vec<Entity>>,
        parent: Option<Entity>,
        transform: Option<&'a Transform>,
        global_transform: Option<&'a GlobalTransform>,
    }

    impl<'a> SceneObjectBuilder<'a> {
        fn new(id: Entity) -> Self {
            Self {
                id,
                name: None,
                children: None,
                parent: None,
                transform: None,
                global_transform: None,
            }
        }
        fn finish(self) -> SceneObject {
            SceneObject {
                id: format!("{:?}", self.id),
                name: self.name.unwrap(),
                children: self
                    .children
                    .map(|children| children.iter().map(|ent| format!("{ent:?}")).collect())
                    .unwrap_or_default(),
                parent: self.parent.map(|ent| format!("{ent:?}")),
                transform: self.transform.unwrap().as_json(),
                global_transform: self.global_transform.unwrap().as_json(),
            }
        }
        fn with_name(mut self, name: String) -> Self {
            self.name = Some(name);
            self
        }
        fn with_children(mut self, children: Vec<Entity>) -> Self {
            self.children = Some(children);
            self
        }
        fn with_parent(mut self, parent: Entity) -> Self {
            self.parent = Some(parent);
            self
        }
        fn with_transform(mut self, transform: &'a Transform) -> Self {
            self.transform = Some(transform);
            self
        }
        fn with_global_transform(mut self, global_transform: &'a GlobalTransform) -> Self {
            self.global_transform = Some(global_transform);
            self
        }
    }

    impl<'a> SceneObject {
        fn build(id: Entity) -> SceneObjectBuilder<'a> {
            SceneObjectBuilder::new(id)
        }
    }

    pub(super) fn notify_entity_spawn_changes(
        mut l_notified: Local<HashSet<Entity>>,
        // Note: We use locals for these data structures purely to reuse
        // existing allocations across multiple invocations of this system
        mut l_new_objects: Local<Vec<SceneObject>>,
        mut l_despawned_entities: Local<Vec<Entity>>,
        mut l_missing_parents: Local<VecDeque<Entity>>,
        q_window: Query<&Window, With<PrimaryWindow>>,
        q_inspectables: Query<(Entity, &Transform, &GlobalTransform), With<Inspectable>>,
        q_others: Query<(Entity, &Transform, &GlobalTransform), Without<Inspectable>>,
        q_names: Query<&Name>,
        q_parents: Query<&Children>,
        q_children: Query<&Parent>,
    ) -> Result<(), String> {
        l_missing_parents.clear();

        l_new_objects.clear();
        l_new_objects.extend(
            q_inspectables
                .iter()
                .filter_map(|(ent, xform, world_xform)| {
                    build_scene_object(
                        &mut l_notified,
                        &mut l_missing_parents,
                        ent,
                        xform,
                        world_xform,
                        &q_inspectables,
                        &q_names,
                        &q_parents,
                        &q_children,
                    )
                }),
        );

        while let Some(parent) = l_missing_parents.pop_front() {
            let Ok((ent, xform, world_xform)) = q_others.get(parent) else {
                error!("Failed to find missing parent: {parent:?}");
                l_notified.insert(parent);

                continue;
            };

            if let Some(object) = build_scene_object(
                &mut l_notified,
                &mut l_missing_parents,
                ent,
                xform,
                world_xform,
                &q_inspectables,
                &q_names,
                &q_parents,
                &q_children,
            ) {
                l_new_objects.push(object);
            }
        }

        l_despawned_entities.clear();
        l_despawned_entities.extend(
            l_notified
                .iter()
                .filter(|ent| !q_inspectables.contains(**ent) && !q_others.contains(**ent))
                .cloned(),
        );

        let win = q_window.single();

        for ent in l_despawned_entities.iter() {
            // TODO: Make this take an array like the multi-spawn event
            wasm::dispatch_to_js(win, "entity-despawn", ent)?;
            l_notified.remove(ent);
        }

        if !l_new_objects.is_empty() {
            // TODO: Remove the single-spawn event
            wasm::dispatch_to_js(win, "entity-multi-spawn", &*l_new_objects)?;
        }

        Ok(())
    }

    fn build_scene_object(
        notified: &mut HashSet<Entity>,
        missing_parents: &mut VecDeque<Entity>,
        ent: Entity,
        xform: &Transform,
        world_xform: &GlobalTransform,
        q_inspectables: &Query<(Entity, &Transform, &GlobalTransform), With<Inspectable>>,
        q_names: &Query<&Name>,
        q_parents: &Query<&Children>,
        q_children: &Query<&Parent>,
    ) -> Option<SceneObject> {
        if notified.contains(&ent) {
            return None;
        }

        notified.insert(ent);

        let mut result = SceneObject::build(ent)
            .with_transform(xform)
            .with_global_transform(world_xform);

        if let Ok(children) = q_parents.get(ent) {
            result = result.with_children(children.iter().cloned().collect());
        }
        if let Ok(parent) = q_children.get(ent) {
            let parent_ent = **parent;
            result = result.with_parent(parent_ent);

            if !notified.contains(&parent_ent) && !q_inspectables.contains(parent_ent) {
                missing_parents.push_back(parent_ent);
            }
        }

        result = result.with_name(match q_names.get(ent) {
            Ok(name) => name.into(),
            Err(_) => format!("{ent:?}"),
        });

        Some(result.finish())
    }
}
