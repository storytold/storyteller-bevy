use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings},
        fxaa::Fxaa,
        tonemapping::Tonemapping,
    },
    gltf::{self, Gltf},
    math::vec2,
    pbr::ShadowFilteringMethod,
    prelude::*,
    render::primitives::Aabb,
    scene::InstanceId,
    utils::HashSet,
};
use bevy_mod_picking::PickableBundle;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    camera_controller::{CameraController, CameraControllerBundle},
    debug::DebugPoint,
    interaction::Selectable,
    math::Extrema,
    MainCamera,
};

pub struct ScenePlugin;

#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SceneSpawnerSystem;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::hsl(0.0, 0.0, 0.25)))
            .insert_resource(Msaa::Off)
            .add_state::<SceneState>()
            .add_event::<LoadSceneEvent>()
            .add_systems(Startup, initialize)
            .add_systems(
                Update,
                (
                    init_scene_loading,
                    tick_scene_loading.run_if(in_state(SceneState::Loading)),
                    tick_scene_spawning.run_if(in_state(SceneState::Spawning)),
                )
                    .in_set(SceneSpawnerSystem),
            )
            .add_systems(
                OnEnter(SceneState::Spawning),
                spawn_next_scene.in_set(SceneSpawnerSystem),
            )
            .add_systems(
                OnEnter(SceneState::Active),
                (tag_scene_elements, frame_scene_in_view).in_set(SceneSpawnerSystem),
            );

        #[cfg(feature = "wasm")]
        app.add_systems(
            Update,
            wasm::notify_state_changes
                .map(bevy::utils::error)
                .run_if(resource_changed::<State<SceneState>>()),
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

#[derive(Event, Deref)]
pub struct LoadSceneEvent(pub String);

// TODOL This could be converted into an enum that indicates what "kind" of
//       element this is (e.g. StaticMesh, SkinnedMesh, Bone, Light, Camera).
//       We don't have an immediate need for that just yet, but likely will
//       soon.
/// A tag that identifies its entity as a user-facing scene element like a prop,
/// a light, or a camera.
#[derive(Component)]
pub struct SceneElement;

fn initialize(mut cmd: Commands) {
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
        SceneElement,
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
        .spawn((Name::new("Scene"), SceneElement, SpatialBundle::default()))
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

fn tag_scene_elements(
    mut cmd: Commands,
    r_scene_spawner: Res<SceneSpawner>,
    r_active_scene: Res<ActiveScene>,
    ra_gltfs: Res<Assets<Gltf>>,
    // A "glTF mesh" is actually a container for one or more "glTF primitives",
    // which is basically a bundle of vertices paired with the material used to
    // shade them. It's those "sub-meshes" that carry the actual mesh handles,
    // but we don't necessarily want to expose those to users a la carte,
    // because they often consist of stuff like a single eyeball, or just the
    // leafy parts of a plant. Here we only want to tag the elements that a user
    // might want to individually address when arranging or animating a scene.
    q_named_non_primitives: Query<&Name, Without<Handle<Mesh>>>,
) {
    let gltf = ra_gltfs.get(&r_active_scene.source).unwrap();

    for ent in r_scene_spawner.iter_instance_entities(**r_active_scene) {
        let Ok(name) = q_named_non_primitives.get(ent) else {
            continue;
        };

        if gltf.named_nodes.contains_key(&**name) {
            cmd.entity(ent).insert(SceneElement);
        }
    }
}

fn frame_scene_in_view(
    mut cmd: Commands,
    r_scene_spawner: Res<SceneSpawner>,
    r_active_scene: Res<ActiveScene>,
    q_aabbs: Query<(&Aabb, &GlobalTransform)>,
    mut q_camera: Query<(&mut CameraController, &mut Transform, &Projection)>,
) {
    let Ok((mut cam, mut cam_xform, cam_proj)) = q_camera.get_single_mut() else {
        return;
    };
    let world2cam = cam_xform.compute_matrix().inverse();
    let aabb_points = r_scene_spawner
        .iter_instance_entities(**r_active_scene)
        .filter_map(|ent| q_aabbs.get(ent).ok())
        .flat_map(|(aabb, world_xform)| {
            let min = world_xform.transform_point(aabb.min().into());
            let max = world_xform.transform_point(aabb.max().into());

            // TODO: The cost of this is probably negligible, but we
            //       should still probably figure out a way to
            //       exclude it from production builds
            for point in Aabb::from_min_max(min, max).extrema() {
                cmd.spawn(DebugPoint {
                    point,
                    color: Color::CYAN,
                });
            }

            [min, max]
        });

    let Some(aabb) = Aabb::enclosing(aabb_points) else {
        return;
    };
    let plane_origin = Vec3::from(aabb.center);
    let plane_normal = cam_xform.forward();

    let Some((_depth, bounding_rect)) = aabb
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
    else {
        return;
    };

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

// TODO: Now that we've tagged all the potentially interesting elements in the
//       scene, we can leave it up to separate plugins to tag and manage the
//       inspectables and selectables.
#[allow(unused, dead_code)]
fn _mark_scene_objects(
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
            cmd.entity(ent_parent).insert(SceneElement);
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

#[cfg(feature = "wasm")]
mod wasm {
    use bevy::{prelude::*, window::PrimaryWindow};
    use wasm_bindgen::prelude::*;

    use super::SceneState;
    use crate::wasm;

    #[wasm_bindgen(typescript_custom_section)]
    const SCENE_STATE_EVENT: &str = r#"
export interface SceneStateEvent extends CustomEvent {
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
}
