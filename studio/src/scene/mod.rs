use bevy::{
    asset::{DependencyLoadState, LoadState},
    core_pipeline::{tonemapping::Tonemapping, Skybox},
    gltf::Gltf,
    math::vec2,
    prelude::*,
    render::primitives::Aabb,
    scene::InstanceId,
    utils::HashSet,
};
#[cfg(feature = "wasm")]
use serde_repr::{Deserialize_repr, Serialize_repr};
use space_editor::{
    prelude::{EditorEvent, EditorPrefabPath, PrefabMarker},
    space_prefab::{
        component::{GltfPrefab, SceneAutoChild},
        editor_registry::EditorRegistryExt,
    },
};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    anim::timeline_animation::AnimationTarget,
    camera_controller::{CameraController, CameraControllerBundle},
    debug::DebugPoint,
    is_editor,
    math::{Combinable, Extrema},
    MainCamera,
};

#[cfg(feature = "headless")]
use crate::screenshot::ScreenCapture;

#[cfg(feature = "wasm")]
mod wasm;

pub struct ScenePlugin;

#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SceneSpawnerSystem;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(feature = "headless"))]
        app.add_plugins(crate::change_tracking::ChangeTrackingPlugin);

        app.insert_resource(ClearColor(Color::hsl(0.0, 0.0, 0.25)))
            .insert_resource(Msaa::Off)
            .init_state::<SceneState>()
            .add_event::<LoadLibraryObjectEvent>()
            .add_event::<LoadSceneEvent>()
            .add_event::<ImportSceneEvent>()
            .add_systems(Startup, initialize)
            .add_systems(
                Update,
                (
                    init_object_loading,
                    init_scene_import,
                    init_scene_loading,
                    #[cfg(feature = "headless")]
                    start_capture,
                )
                    .in_set(SceneSpawnerSystem),
            )
            .add_systems(
                OnEnter(SceneState::Active),
                (tag_scene_elements, frame_scene_in_view)
                    .chain()
                    .in_set(SceneSpawnerSystem),
            );

        app.add_systems(
            Update,
            (
                tag_scene_elements,
                autoplay_animations.run_if(not(is_editor)),
            ),
        );

        app.init_resource::<SkyboxHolder>();
        app.add_systems(Update, attach_skybox);

        //Auto loading screen
        //Register handles on which we will look to show loading screen
        app.init_resource::<LoadingQueue>();
        app.add_systems(
            PostUpdate,
            (
                auto_add_handle::<Scene>,
                auto_add_handle::<Mesh>,
                auto_add_handle::<StandardMaterial>,
            ),
        );

        app.add_systems(PreUpdate, check_loading);

        //setup default light
        app.add_systems(Startup, minimal_scene_setup);

        #[cfg(feature = "wasm")]
        app.add_systems(
            Update,
            (
                wasm::handle_spawn_asset_messages,
                wasm::notify_state_changes
                    .map(bevy::utils::error)
                    .run_if(resource_changed::<State<SceneState>>),
                wasm::notify_loading_queue_changes.map(bevy::utils::error),
            ),
        );

        app.editor_registry::<SceneElement>();
    }
}

#[derive(Resource, Default)]
pub struct SkyboxHolder(pub Option<Skybox>);

fn attach_skybox(
    mut cmd: Commands,
    skybox_holder: ResMut<SkyboxHolder>,
    mut q_camera: Query<Entity, (With<Camera3d>, Without<Skybox>)>,
) {
    let Some(skybox) = skybox_holder.0.as_ref() else {
        return;
    };

    for entity in q_camera.iter_mut() {
        cmd.entity(entity).insert(skybox.clone());
    }
}

fn minimal_scene_setup(mut cmd: Commands) {
    cmd.spawn((
        Name::new("Directional Light"),
        AnimationTarget::new(),
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                shadows_enabled: false,
                illuminance: 5000.0,
                ..default()
            },
            transform: Transform::default().looking_at(Vec3::NEG_ONE, Vec3::Y),
            ..default()
        },
        PrefabMarker,
        SceneElement::Light,
    ));
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(States, Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub enum SceneState {
    #[default]
    None,
    Loading,
    Active,
}

#[derive(Resource, Deref)]
pub struct ActiveScene {
    #[deref]
    pub instance: InstanceId,
    pub source: Handle<Gltf>,
}

#[derive(Resource, Default)]
pub struct LoadingQueue {
    pub queue: Vec<UntypedHandle>,
}

fn check_loading(
    mut frames_without_loading: Local<usize>,
    mut loading_queue: ResMut<LoadingQueue>,
    mut st_next: ResMut<NextState<SceneState>>,
    r_scene_state: Res<State<SceneState>>,
    asset_server: Res<AssetServer>,
) {
    // Avoid transitioning from `SceneState::None` directly to `SceneState::Active`.
    // We will always be initializing the scene with _something_, so if the
    // `LoadingQueue` is empty when we're in `SceneState::None`, it just means
    // our startup routine hasn't yet gotten to the point of processing any
    // `Handle<T>`s.
    if loading_queue.queue.is_empty()
        && **r_scene_state == SceneState::Loading
        && *frames_without_loading > 200
    {
        info!("Transition to active");
        st_next.set(SceneState::Active);
        return;
    }

    if loading_queue.queue.is_empty() && **r_scene_state == SceneState::Loading {
        *frames_without_loading += 1;
    }

    if !loading_queue.queue.is_empty() {
        *frames_without_loading = 0;
        //filter queue to only include notloaded assets

        let mut not_loaded_count = 0;

        // info!("Loading {} assets", loading_queue.queue.len());

        loading_queue.queue.retain(|h| {
            if let Some((load_state, dep_load_state, _)) = asset_server.get_load_states(h.id()) {
                if load_state == LoadState::NotLoaded {
                    not_loaded_count += 1;
                }

                load_state == LoadState::Loading || dep_load_state == DependencyLoadState::Loading
            } else {
                false
            }
        });

        if loading_queue.queue.len() - not_loaded_count == 0 {
            //not change state
        } else {
            st_next.set(SceneState::Loading);
        }
    }
}

fn auto_add_handle<T: Asset>(
    mut queue: ResMut<LoadingQueue>,
    query: Query<&Handle<T>, Changed<Handle<T>>>,
) {
    for h in query.iter() {
        info!("Added: {}", h.id());
        queue.queue.push(h.clone().untyped());
    }
}

/// Request the application to load a `gltf`/`glb` scene from the "bundled"
/// asset library in `assets/gltf/`. Expects a filename including the extension.
///
/// ```ignore
/// fn main() {
///     App::new()
///         .add_plugins((
///             DefaultPlugins,
///             CommonPlugins,
///         ))
///         .add_systems(Startup, load_mannequin)
/// }
///
/// fn load_mannequin(mut ew_load_object: EventWriter<LoadLibraryObjectEvent>) {
///     ew_load_object.send(LoadLibraryObjectEvent("Mannequin.gltf".into()));
/// }
/// ```
#[derive(Event, Deref)]
pub struct LoadLibraryObjectEvent(pub String);

/// Request the application to load a previously saved Storyteller Studio scene.
///
/// Expects a full, resolvable path to the file (e.g. `scenes/my-scene.scn.ron`
/// for a local file saved under `studio/assets/scenes`, or
/// `remote://<media_token>.scn.ron` for a scene saved to the Storyteller.ai
/// backend).
#[derive(Event, Deref)]
pub struct LoadSceneEvent(pub String);

/// Request the application to import and process a glTF scene from an arbitrary
/// asset source (e.g., an http path or a Storyteller.ai media token).
#[derive(Event)]
pub struct ImportSceneEvent {
    pub path: String,
    pub name: Option<String>,
}

/// A tag that identifies its entity as a user-facing scene element like a prop,
/// a light, or a camera.
#[cfg_attr(
    feature = "wasm",
    repr(u8),
    wasm_bindgen,
    derive(Serialize_repr, Deserialize_repr)
)]
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Reflect, Default)]
#[reflect(Component, Default)]
#[allow(dead_code)] // Some variants only used in `wasm`, some planned for later use
pub enum SceneElement {
    #[default]
    Generic,
    Mesh,
    Skeleton,
    Bone,
    Light,
    Camera,
}

#[derive(Component)]
pub struct SceneData {
    pub scene_path: String,
}

#[derive(Component)]
pub struct SceneRoot;

#[cfg(not(feature = "headless"))]
fn initialize(mut cmd: Commands) {
    use crate::change_tracking::ChangeTracker;
    use bevy::{
        core_pipeline::{
            bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings},
            fxaa::Fxaa,
        },
        pbr::ShadowFilteringMethod,
    };

    // Camera, graphics settings
    let focus = Vec3::ZERO;
    let camera_xform = Transform::from_xyz(-2.5, 4.5, 5.0).looking_at(focus, Vec3::Y);

    cmd.spawn((
        Name::new("Main Camera"),
        MainCamera,
        AnimationTarget::MainCamera,
        ChangeTracker,
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
        // FIXME: This should not be the user-visible/keyframable camera
        SceneElement::Camera,
    ));
}

#[cfg(feature = "headless")]
fn initialize(
    mut cmd: Commands,
    mut image_assets: ResMut<Assets<Image>>,
    headless_settings: Res<crate::headless::HeadlessSettings>,
) {
    use bevy::render::camera::RenderTarget;
    use wgpu::{TextureDescriptor, TextureUsages};

    // Camera, graphics settings
    let focus = Vec3::ZERO;
    let camera_xform = Transform::from_xyz(-2.5, 4.5, 5.0).looking_at(focus, Vec3::Y);

    let mut test_image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("render_texture"),
            size: headless_settings.output_dims,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..Default::default()
    };

    test_image.resize(headless_settings.output_dims);

    cmd.spawn((
        Name::new("Main Camera"),
        MainCamera,
        AnimationTarget::MainCamera,
        Camera3dBundle {
            transform: camera_xform,
            tonemapping: Tonemapping::BlenderFilmic,
            camera: Camera {
                hdr: false,
                target: RenderTarget::Image(image_assets.add(test_image)),
                ..Default::default()
            },
            ..default()
        },
        CameraControllerBundle::new(focus, camera_xform.translation),
    ));
}

fn init_object_loading(mut cmd: Commands, mut er: EventReader<LoadLibraryObjectEvent>) {
    if let Some(id) = er.read().last() {
        info!("Loading library asset: {}", id.0);
        cmd.spawn((
            Name::new(id.0.clone()),
            SpatialBundle::default(),
            space_editor::prelude::GltfPrefab {
                path: format!("gltf/{}", id.0),
                scene: "Scene0".to_string(),
            },
            PrefabMarker,
            SceneRoot,
        ));
    }
}

fn init_scene_import(
    mut cmd: Commands,
    r_assets: Res<AssetServer>,
    mut er: EventReader<ImportSceneEvent>,
) {
    for event in er.read() {
        info!("Importing asset: {}", event.path);

        let name = event
            .name
            .as_ref()
            .map(|name| Name::new(name.clone()))
            .unwrap_or_else(|| Name::new("Scene"));

        cmd.spawn((
            name,
            SpatialBundle::default(),
            space_editor::prelude::GltfPrefab {
                path: event.path.clone(),
                scene: "Scene0".to_string(),
            },
            r_assets.load::<Gltf>(&event.path),
            PrefabMarker,
            SceneRoot,
        ));
    }
}

fn init_scene_loading(
    mut er_load_scene: EventReader<LoadSceneEvent>,
    mut ew_editor: EventWriter<EditorEvent>,
) {
    if let Some(scene_path) = er_load_scene.read().last() {
        info!("Loading scene: {}", scene_path.0);
        ew_editor.send(EditorEvent::Load(EditorPrefabPath::File(
            scene_path.0.clone(),
        )));
    }
}

fn tag_scene_elements(
    mut cmd: Commands,
    // A "glTF mesh" is actually a container for one or more "glTF primitives",
    // which is basically a bundle of vertices paired with the material used to
    // shade them. It's those "sub-meshes" that carry the actual mesh handles,
    // but we don't necessarily want to expose those to users a la carte,
    // because they often consist of stuff like a single eyeball, or just the
    // leafy parts of a plant. Here we only want to tag the elements that a user
    // might want to individually address when arranging or animating a scene.
    q_named_non_primitives: Query<
        (Entity, &Children),
        (
            With<SceneAutoChild>,
            With<Name>,
            Without<Handle<Mesh>>,
            Without<SceneElement>,
        ),
    >,
    mesh_entities: Query<Entity, With<Handle<Mesh>>>,
) {
    for (e, children) in q_named_non_primitives.iter() {
        if children.iter().any(|child| mesh_entities.contains(*child)) {
            cmd.entity(e).insert(SceneElement::Mesh);
        }
    }
}

fn autoplay_animations(
    mut cmd: Commands,
    mut l_done: Local<HashSet<String>>,
    mut l_animating: Local<HashSet<Entity>>,
    ra_gltfs: Res<Assets<Gltf>>,
    ra_anims: Res<Assets<AnimationClip>>,
    q_prefabs: Query<(&GltfPrefab, &Handle<Gltf>)>,
    q_named_ents: Query<(Entity, &Name)>,
    mut q_players: Query<&mut AnimationPlayer>,
) {
    for (prefab, handle) in q_prefabs.iter() {
        if l_done.contains(&prefab.path) {
            continue;
        }

        let Some(gltf) = ra_gltfs.get(handle) else {
            continue;
        };

        let mut playing_anims = 0_usize;
        for (anim_name, anim_handle) in gltf.named_animations.iter() {
            let Some(anim) = ra_anims.get(anim_handle.clone()) else {
                continue;
            };

            let Some((target, name)) = q_named_ents
                .iter()
                .find(|(_, name)| anim.compatible_with(name))
            else {
                continue;
            };

            if !l_animating.contains(&target) {
                info!(
                    "Playing animation '{}' on entity {:?} '{}'",
                    anim_name, target, name,
                );
                if let Ok(mut player) = q_players.get_mut(target) {
                    if !player.is_playing_clip(anim_handle) {
                        player.play(anim_handle.clone()).repeat();
                    }
                } else {
                    let mut player = AnimationPlayer::default();
                    player.play(anim_handle.clone()).repeat();
                    cmd.entity(target).insert(player);
                }
                l_animating.insert(target);
            }

            playing_anims += 1;
        }

        if playing_anims == gltf.named_animations.len() {
            l_done.insert(prefab.path.clone());
        }
    }
}

fn frame_scene_in_view(
    mut cmd: Commands,
    q_aabbs: Query<(&Aabb, &GlobalTransform), With<SceneAutoChild>>,
    mut q_camera: Query<(&mut CameraController, &mut Transform, &Projection)>,
) {
    let Ok((mut cam, mut cam_xform, cam_proj)) = q_camera.get_single_mut() else {
        error!("Failed to get camera controller");
        return;
    };

    let Some(aabb) = Aabb::combine(q_aabbs.iter()) else {
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
            let proj = p - v.project_onto_normalized(plane_normal.xyz());

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

#[cfg(feature = "headless")]
fn start_capture(mut captures: Query<&mut ScreenCapture, Added<ScreenCapture>>) {
    for mut capture in captures.iter_mut() {
        capture.start_capturing();
    }
}
