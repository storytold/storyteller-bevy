use bevy::{
    asset::AssetPath,
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings},
        fxaa::Fxaa,
        tonemapping::Tonemapping,
    },
    gltf::Gltf,
    math::{vec2, vec3},
    pbr::ShadowFilteringMethod,
    prelude::*,
    render::{
        mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
        primitives::Aabb,
        view::NoFrustumCulling,
    },
    scene::InstanceId,
};

use space_editor::prelude::{space_undo::OneFrameUndoIgnore, PrefabMarker};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    bvh::{Bvh, CommandsSpawnBvhExt},
    camera_controller::{CameraController, CameraControllerBundle},
    debug::DebugPoint,
    math::{Combinable, Extrema},
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
                    spawn_test_bvh, // TODO
                    init_scene_loading,
                    tick_scene_loading.run_if(in_state(SceneState::Loading)),
                    tick_scene_spawning.run_if(in_state(SceneState::Spawning)),
                )
                    .in_set(SceneSpawnerSystem),
            )
            .add_systems(
                OnEnter(SceneState::Loading),
                despawn_previous_scene.in_set(SceneSpawnerSystem),
            )
            .add_systems(
                OnEnter(SceneState::Spawning),
                spawn_next_scene.in_set(SceneSpawnerSystem),
            )
            .add_systems(
                OnEnter(SceneState::Active),
                (
                    tag_scene_elements,
                    spawn_main_scene_light,
                    frame_scene_in_view,
                )
                    .in_set(SceneSpawnerSystem),
            );

        #[cfg(feature = "wasm")]
        app.add_systems(
            Update,
            (
                wasm::dispatch_scene_request,
                wasm::notify_state_changes
                    .map(bevy::utils::error)
                    .run_if(resource_changed::<State<SceneState>>()),
            ),
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
pub struct ActiveScene {
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

#[derive(Component)]
pub struct SkeletalJoint(pub Color);

#[derive(Component)]
pub struct SceneRoot;

#[derive(Resource, Deref, DerefMut)]
struct TestBvh(Handle<Bvh>); // TODO

#[derive(Component, Clone, Copy, Debug, Deref, DerefMut)]
pub struct BindPoseLocal(pub Transform);

#[derive(Component, Clone, Copy, Debug, Deref, DerefMut)]
pub struct BindPoseGlobal(pub Mat4);

#[derive(Component, Clone, Copy, Debug, Deref, DerefMut)]
pub struct InvBindPose(pub Mat4);

fn initialize(mut cmd: Commands, r_assets: Res<AssetServer>) {
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

    // TODO
    cmd.insert_resource(TestBvh(r_assets.load("mocap/shuffle.bvh")));
}

// TODO
fn spawn_test_bvh(
    mut commands: Commands,
    mut er_bvh_events: EventReader<AssetEvent<Bvh>>,
    r_bvh: Res<TestBvh>,
) {
    for event in er_bvh_events.read() {
        if matches!(event, AssetEvent::LoadedWithDependencies { .. }) {
            commands.spawn_bvh_with_transform(
                &r_bvh.0,
                Transform::from_scale(Vec3::splat(0.06))
                    .with_translation(Vec3::new(0., 1.0, 0.524)),
            );
        }
    }
}

fn init_scene_loading(
    mut cmd: Commands,
    assets: Res<AssetServer>,
    r_active_scene: Option<Res<ActiveScene>>,
    mut er: EventReader<LoadSceneEvent>,
    mut st_next: ResMut<NextState<SceneState>>,
) {
    if let Some(id) = er.read().last() {
        let name = &**id;
        let path = format!("gltf/{name}.glb");

        // Avoid trying to re-load this scene if it's already active
        if let Some(active_scene_path) =
            r_active_scene.and_then(|scene| scene.source.path().cloned())
        {
            if active_scene_path == AssetPath::parse(&path) {
                return;
            }
        }

        cmd.insert_resource(PendingScene(assets.load(path)));
        st_next.set(SceneState::Loading);
    }
}

fn despawn_previous_scene(mut cmd: Commands, q_scene_root: Query<Entity, With<SceneRoot>>) {
    if let Ok(scene_root) = q_scene_root.get_single() {
        cmd.entity(scene_root).despawn_recursive();
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
        .spawn((
            Name::new("Scene"),
            SceneRoot,
            SceneElement,
            SpatialBundle::default(),
            PrefabMarker,
            OneFrameUndoIgnore::default(), //To prevert undo try to register add event
        ))
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
    ra_inv_bindposes: Res<Assets<SkinnedMeshInverseBindposes>>,
    // A "glTF mesh" is actually a container for one or more "glTF primitives",
    // which is basically a bundle of vertices paired with the material used to
    // shade them. It's those "sub-meshes" that carry the actual mesh handles,
    // but we don't necessarily want to expose those to users a la carte,
    // because they often consist of stuff like a single eyeball, or just the
    // leafy parts of a plant. Here we only want to tag the elements that a user
    // might want to individually address when arranging or animating a scene.
    q_named_non_primitives: Query<&Name, Without<Handle<Mesh>>>,
    q_skinned_meshes: Query<&SkinnedMesh>,
    mut q_xforms: ParamSet<(TransformHelper, Query<&Transform>)>,
) {
    let gltf = ra_gltfs.get(&r_active_scene.source).unwrap();

    for (ent, name) in r_scene_spawner
        .iter_instance_entities(**r_active_scene)
        .filter_map(|ent| match q_named_non_primitives.get(ent) {
            Ok(name) => Some((ent, name)),
            _ => None,
        })
    {
        if gltf.named_nodes.contains_key(&**name) {
            cmd.entity(ent).insert(SceneElement).insert(PrefabMarker);
        }
    }

    for (ent, skinned_mesh) in r_scene_spawner
        .iter_instance_entities(**r_active_scene)
        .filter_map(|ent| match q_skinned_meshes.get(ent) {
            Ok(skinned_mesh) => Some((ent, skinned_mesh)),
            _ => None,
        })
    {
        cmd.entity(ent).insert(NoFrustumCulling);

        let inv_bind_poses = ra_inv_bindposes
            .get(skinned_mesh.inverse_bindposes.clone())
            .unwrap();

        for (idx, &joint) in skinned_mesh.joints.iter().enumerate() {
            let inv_bind_pose = inv_bind_poses[idx];
            let global_bind_pose = q_xforms.p0().compute_global_transform(joint).unwrap();
            let local_bind_pose = *q_xforms.p1().get(joint).unwrap();

            cmd.entity(joint).insert((
                SkeletalJoint(Color::BLACK),
                BindPoseLocal(local_bind_pose),
                BindPoseGlobal(global_bind_pose.compute_matrix()),
                InvBindPose(inv_bind_pose),
            ));
        }
    }
}

fn spawn_main_scene_light(
    mut cmd: Commands,
    r_scene_spawner: Res<SceneSpawner>,
    r_active_scene: Res<ActiveScene>,
    q_aabbs: Query<(&Aabb, &GlobalTransform)>,
    q_scene_root: Query<Entity, With<SceneRoot>>,
) {
    let Ok(scene_root) = q_scene_root.get_single() else {
        return;
    };

    let Some(aabb) = Aabb::combine(
        r_scene_spawner
            .iter_instance_entities(**r_active_scene)
            .filter_map(|ent| q_aabbs.get(ent).ok()),
    ) else {
        return;
    };

    let light_offset_base = vec3(0.5, 1., 1.).normalize();
    let size = f32::max(
        f32::max(aabb.half_extents.x, aabb.half_extents.y),
        aabb.half_extents.z,
    ) * 2.;
    let light_pos = Vec3::from(aabb.center) + light_offset_base * size * 2.;

    cmd.spawn(DebugPoint {
        point: light_pos,
        color: Color::WHITE,
    });
    cmd.entity(scene_root).with_children(|cmd| {
        cmd.spawn((
            Name::new("Light"),
            SceneElement,
            SpotLightBundle {
                spot_light: SpotLight {
                    color: Color::WHITE,
                    // TODO: This doesn't scale super well to very large objects
                    intensity: 200. * size,
                    radius: 0.1,
                    shadows_enabled: true,
                    shadow_normal_bias: 0.2 * size,
                    outer_angle: f32::to_radians(150.),
                    inner_angle: f32::to_radians(45.),
                    ..default()
                },
                transform: Transform::from_translation(light_pos)
                    .looking_at(aabb.center.into(), Vec3::Y),
                ..default()
            },
        ));
    });
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

    let Some(aabb) = Aabb::combine(
        r_scene_spawner
            .iter_instance_entities(**r_active_scene)
            .filter_map(|ent| q_aabbs.get(ent).ok()),
    ) else {
        return;
    };

    let world2cam = cam_xform.compute_matrix().inverse();
    let plane_origin = Vec3::from(aabb.center);
    let plane_normal = cam_xform.forward();

    let Some((_depth, bounding_rect)) = aabb
        .extrema()
        .iter()
        .copied()
        .map(|mut p| {
            cmd.spawn(DebugPoint {
                point: p,
                color: Color::CYAN,
            });

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

#[cfg(feature = "wasm")]
mod wasm {
    use bevy::{prelude::*, window::PrimaryWindow};
    use wasm_bindgen::prelude::*;

    use super::{LoadSceneEvent, SceneState};
    use crate::wasm::{self, StaticBuffer, WasmMessageBuffer};

    static OBJECT_LOAD_REQUEST: WasmMessageBuffer<String> = WasmMessageBuffer::new();

    #[wasm_bindgen(js_name = loadObject)]
    pub fn load_object(id: &str) {
        OBJECT_LOAD_REQUEST.write_message(id.into());
    }

    // FIXME: Is it an object or a scene? Inconsistent terminology is
    //        unnecessarily confusing
    pub(super) fn dispatch_scene_request(mut ew_load_scene: EventWriter<LoadSceneEvent>) {
        if let Some(id) = OBJECT_LOAD_REQUEST.take_message() {
            ew_load_scene.send(LoadSceneEvent(id));
        }
    }

    #[wasm_bindgen(typescript_custom_section)]
    const SCENE_STATE_EVENT: &str = r#"
export interface SceneStateEvent extends CustomEvent {
    type: "scene-state";
    detail: SceneState;
}

declare global {
    export interface HTMLElementEventMap {
        "scene-state": SceneStateEvent;
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
