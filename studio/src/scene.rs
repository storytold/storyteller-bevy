use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings},
        fxaa::Fxaa,
        tonemapping::Tonemapping,
    },
    gltf::{self, Gltf},
    math::vec2,
    pbr::{NotShadowCaster, ShadowFilteringMethod},
    prelude::*,
    reflect::TypePath,
    render::{
        primitives::Aabb,
        render_resource::{AsBindGroup, ShaderRef},
    },
    scene::InstanceId,
    utils::{HashMap, HashSet},
};
use bevy_mod_picking::PickableBundle;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    camera_controller::{CameraController, CameraControllerBundle, CameraMotionSystem},
    debug::DebugPoint,
    interaction::Selectable,
    math::Extrema,
    skybox::SkyboxId,
    MainCamera,
};

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::hsl(0.0, 0.0, 0.25)))
            .insert_resource(Msaa::Off)
            .add_state::<SceneState>()
            .add_event::<LoadSceneEvent>()
            .add_plugins(MaterialPlugin::<ProceduralGridMaterial>::default())
            .add_systems(Startup, initialize)
            .add_systems(
                Update,
                (
                    init_scene_loading,
                    tick_scene_loading.run_if(in_state(SceneState::Loading)),
                    tick_scene_spawning.run_if(in_state(SceneState::Spawning)),
                ),
            )
            .add_systems(OnEnter(SceneState::Spawning), spawn_next_scene)
            .add_systems(OnEnter(SceneState::Active), mark_scene_objects)
            .add_systems(PostUpdate, move_plane_with_camera.after(CameraMotionSystem));

        #[cfg(feature = "wasm")]
        app.add_systems(
            Update,
            wasm::notify_state_changes
                .map(bevy::utils::error)
                .run_if(resource_changed::<State<SceneState>>()),
        )
        .add_systems(
            PostUpdate,
            wasm::notify_entity_spawn_changes.map(bevy::utils::error),
        );
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(States, Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub enum SceneState {
    #[default]
    None,
    Loading,
    Spawning,
    Active,
}

#[derive(Resource, Deref, DerefMut)]
struct PendingScene(pub Handle<Gltf>);

#[derive(Resource, Deref)]
struct ActiveScene {
    #[deref]
    pub instance: InstanceId,
    pub source: Handle<Gltf>,
}

#[derive(Component)]
struct WorldGridPlane;

#[derive(Component)]
struct Inspectable;

#[derive(Resource, Deref, DerefMut)]
pub struct ObjectLibrary(pub HashMap<String, Handle<Gltf>>);

#[derive(Event, Deref)]
pub struct LoadSceneEvent(pub String);

fn initialize(
    mut cmd: Commands,
    mut ra_meshes: ResMut<Assets<Mesh>>,
    mut ra_mat_proc_grid: ResMut<Assets<ProceduralGridMaterial>>,
) {
    // Skybox
    cmd.insert_resource(SkyboxId("test_scene".into()));

    // glTF scene
    // cmd.insert_resource(NewGltf(assets.load("gltf/sample-room.gltf")));

    // Camera, graphics settings
    let focus = Vec3::ZERO;
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
        ShadowFilteringMethod::Hardware2x2,
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
    // TODO: Figure out a scene-agnostic lighting setup
    cmd.spawn((
        Name::new("Light"),
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

fn tick_scene_loading(
    r_pending_scene: Res<PendingScene>,
    ra_gltfs: Res<Assets<Gltf>>,
    mut er_scene_events: EventReader<AssetEvent<Scene>>,
    mut st_next: ResMut<NextState<SceneState>>,
) {
    for event in er_scene_events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            let gltf = ra_gltfs.get(&**r_pending_scene).unwrap();
            if gltf.default_scene.is_some() && gltf.default_scene.as_ref().unwrap().id() == *id {
                st_next.set(SceneState::Spawning);
                break;
            }
        }
    }
}

fn spawn_next_scene(
    mut cmd: Commands,
    ra_gltfs: Res<Assets<Gltf>>,
    r_pending_scene: Res<PendingScene>,
    mut r_scene_spawner: ResMut<SceneSpawner>,
) {
    let gltf = ra_gltfs.get(&**r_pending_scene).unwrap();
    let scene = gltf
        .default_scene
        .as_ref()
        .expect("Expected glTF to include a default scene");

    let parent = cmd
        .spawn((Name::new("Scene"), Inspectable, SpatialBundle::default()))
        .id();

    let instance = r_scene_spawner.spawn_as_child(scene.clone(), parent);

    cmd.remove_resource::<PendingScene>();
    cmd.insert_resource(ActiveScene {
        instance,
        source: r_pending_scene.clone(),
    });
}

fn tick_scene_spawning(
    r_scene_spawner: Res<SceneSpawner>,
    r_active_scene: Res<ActiveScene>,
    mut st_next: ResMut<NextState<SceneState>>,
) {
    // TODO: The instance is considered "ready" even if shaders still need to be
    //       compiled, so this event isn't really sufficient for controlling a
    //       loading screen.
    if r_scene_spawner.instance_is_ready(**r_active_scene) {
        st_next.set(SceneState::Active);
    }
}

fn mark_scene_objects(
    mut cmd: Commands,
    r_scene: Res<ActiveScene>,
    ra_gltfs: Res<Assets<Gltf>>,
    ra_gltf_nodes: Res<Assets<gltf::GltfNode>>,
    ra_gltf_meshes: Res<Assets<gltf::GltfMesh>>,
    q_parents: Query<(Entity, &Name, &Children), Without<Selectable>>,
    q_primitives: Query<&Handle<Mesh>>,
    // FIXME: This belongs in a different system
    q_aabbs: Query<(Entity, &Aabb, &Transform, &Parent), Without<CameraController>>,
    q_parent_xforms: Query<(&Transform, Option<&Parent>), Without<CameraController>>,
    mut q_camera: Query<(&mut CameraController, &mut Transform, &Projection)>,
) {
    let gltf = ra_gltfs.get(&r_scene.source).unwrap();

    let mut scene_primitives = HashSet::<Entity>::new();
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

        // Collect all of the mesh handles in the glTF mesh
        let prims_from_gltf = gltf_mesh
            .primitives
            .iter()
            .filter_map(|prim| prim.mesh.path())
            .collect::<HashSet<_>>();

        // Collect all of the mesh handles in this entity's children
        let prims_from_ecs = children
            .iter()
            .filter_map(|ent_child| {
                q_primitives
                    .get(*ent_child)
                    .ok()
                    .and_then(|handle| handle.path())
            })
            .collect::<HashSet<_>>();

        // If the sets match, this is the entity that represents the glTF mesh
        if prims_from_gltf == prims_from_ecs {
            cmd.entity(ent_parent).insert(Selectable);

            for &ent_child in children.iter() {
                if q_primitives.contains(ent_child) {
                    scene_primitives.insert(ent_child);
                    cmd.entity(ent_child).insert(PickableBundle::default());
                }
            }
        }
    }

    // FIXME: This belongs in a different system
    if let Ok((mut cam, mut cam_xform, cam_proj)) = q_camera.get_single_mut() {
        let world2cam = cam_xform.compute_matrix().inverse();
        let aabb_points = q_aabbs
            .iter()
            // Filter for AABBs belonging to the glTF scene
            .filter(|(ent, _, _, _)| scene_primitives.contains(ent))
            // Transform the AABBs into world-space
            .flat_map(|(_, aabb, xform, parent)| {
                let mut min = xform.transform_point(aabb.min().into());
                let mut max = xform.transform_point(aabb.max().into());
                let mut parent = Some(parent);
                loop {
                    if let Some((parent_xform, parent_parent)) =
                        parent.and_then(|parent| q_parent_xforms.get(**parent).ok())
                    {
                        min = parent_xform.transform_point(min);
                        max = parent_xform.transform_point(max);
                        parent = parent_parent;
                    } else {
                        // TODO: The cost of this is probably negligible, but we
                        //       should still probably figure out a way to
                        //       exclude it from production builds
                        for point in Aabb::from_min_max(min, max).extrema() {
                            cmd.spawn(DebugPoint {
                                point,
                                color: Color::CYAN,
                            });
                        }

                        break [min, max];
                    }
                }
            });

        if let Some(aabb) = Aabb::enclosing(aabb_points) {
            let plane_origin = Vec3::from(aabb.center);
            let plane_normal = cam_xform.forward();

            if let Some((_depth, bounding_rect)) = aabb
                .extrema()
                .iter()
                .copied()
                .map(|mut p| {
                    // Inflate the combined AABB to give the model some breathing room
                    p *= 1.5;

                    // Project each corner onto the camera plane
                    let v = p - plane_origin;
                    let proj = p - v.project_onto_normalized(plane_normal);
                    cmd.spawn(DebugPoint {
                        point: proj,
                        color: Color::FUCHSIA,
                    });

                    // Transform into the camera's local space
                    world2cam.transform_point(proj)
                })
                // Fold the camera-space plane-projected points into a bounding rect and view depth
                .fold(None, |accum: Option<(f32, Rect)>, p: Vec3| match accum {
                    Some((depth, rect)) => Some((depth, rect.union_point(vec2(p.x, p.y)))),
                    None => {
                        let depth = p.z.abs();
                        let p = vec2(p.x, p.y);
                        Some((depth, Rect::from_corners(p, p)))
                    }
                })
            {
                let pproj = match cam_proj {
                    Projection::Perspective(p) => p,
                    _ => panic!("Expected a perspective projection"),
                };

                // Formula for the target camera distance
                // (c/o https://stackoverflow.com/questions/2866350/move-camera-to-fit-3d-scene):
                //
                // d = (s/2) / tan(a/2)
                // where:
                //   d = distance
                //   s = size of the largest camera-aligned axis in world coords
                //   a = field-of-view angle for the same axis
                let Vec2 {
                    x: width,
                    y: height,
                } = bounding_rect.size();

                let s = f32::max(width, height);
                let a = if s == height {
                    pproj.fov
                } else {
                    f32::atan(pproj.aspect_ratio * f32::tan(pproj.fov / 2.))
                };

                // TODO:
                // According to "I eat babies" at the Ars Technica forums
                // (https://arstechnica.com/civis/threads/if-you-have-aspect-ratio-and-vertical-fov-can-you-calc-horizontal-fov.37447/):
                //
                // tan(fov_v / 2) = y / z
                // tan(fov_h / 2) = x / z
                //
                // Theoretically, this would let us skip some costly tan/atan
                // computations, but I must be missing something, because just
                // substituting (s / depth) for the divisor below would make the
                // whole equation independent of the FOV, which is obviously
                // wrong.
                //
                // Unfortunately I'm embarrassingly bad at algebra, but there
                // must be some room for simplification here.
                let radius = (s / 2.) / f32::tan(a / 2.);

                cam.focus = aabb.center.into();
                cam.radius = radius;
                cam_xform.translation = cam.focus + cam_xform.back() * radius;
            }
        }
    }
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

fn init_scene_loading(
    mut cmd: Commands,
    assets: Res<AssetServer>,
    mut er: EventReader<LoadSceneEvent>,
    mut st_next: ResMut<NextState<SceneState>>,
) {
    if let Some(id) = er.read().last() {
        let name = &**id;
        cmd.insert_resource(PendingScene(assets.load(format!("gltf/{name}.gltf"))));
        st_next.set(SceneState::Loading);
    }
}

#[derive(Asset, AsBindGroup, TypePath, Clone)]
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

    use super::{Inspectable, SceneState};

    #[wasm_bindgen(typescript_custom_section)]
    const SCENE_STATE_EVENT: &str = r#"
export interface SceneStateEvent {
    type: "scene-state";
    detail: SceneState;
}

declare global {
    export interface HTMLElementEventMap {
        "scene_state": SceneStateEvent;
    }
}
    "#;

    pub(super) fn notify_state_changes(
        r_state: Res<State<SceneState>>,
        q_window: Query<&Window, With<PrimaryWindow>>,
    ) -> Result<(), String> {
        if let Ok(window) = q_window.get_single() {
            wasm::dispatch_to_js(window, "scene-state", &(**r_state as u32))?;
        }
        Ok(())
    }

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
