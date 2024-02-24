use bevy::{
    asset::{DependencyLoadState, LoadState, RecursiveDependencyLoadState},
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings},
        fxaa::Fxaa,
        tonemapping::Tonemapping,
    },
    gltf::Gltf,
    math::vec2,
    pbr::ShadowFilteringMethod,
    prelude::*,
    render::{camera::RenderTarget, primitives::Aabb},
    scene::InstanceId,
};

use space_editor::{
    prelude::{EditorEvent, EditorPrefabPath, PrefabMarker},
    space_prefab::component::SceneAutoChild,
};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;
use wgpu::{TextureDescriptor, TextureUsages};

use crate::{
    camera_controller::{CameraController, CameraControllerBundle},
    debug::DebugPoint,
    is_headless, is_not_headless,
    math::{Combinable, Extrema},
    screenshot::ScreenCapture,
    HeadlessSettings, MainCamera,
};

pub struct ScenePlugin;

#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SceneSpawnerSystem;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::hsl(0.0, 0.0, 0.25)))
            .insert_resource(Msaa::Off)
            .add_state::<SceneState>()
            .add_event::<LoadLibraryObjectEvent>()
            .add_event::<LoadSceneEvent>()
            .add_systems(
                Startup,
                (
                    initialize.run_if(is_not_headless),
                    initialiaze_headless.run_if(is_headless),
                ),
            )
            .add_systems(
                Update,
                (init_object_loading, init_scene_loading).in_set(SceneSpawnerSystem),
            )
            .add_systems(
                OnEnter(SceneState::Active),
                (
                    tag_scene_elements,
                    frame_scene_in_view,
                    start_capture.run_if(is_headless),
                )
                    .in_set(SceneSpawnerSystem),
            );

        app.add_systems(Update, tag_scene_elements);

        //Auto loading screen
        //Register handles on which we will look to show loading screen
        app.init_resource::<LoadingQueue>();
        app.add_systems(
            PostUpdate,
            (
                check_loading,
                auto_add_handle::<Scene>,
                auto_add_handle::<Mesh>,
                auto_add_handle::<StandardMaterial>,
            ),
        );

        //setup default light
        app.add_systems(Startup, minimal_scene_setup);

        #[cfg(feature = "wasm")]
        app.add_systems(
            Update,
            (
                wasm::dispatch_object_load_request,
                wasm::notify_state_changes
                    .map(bevy::utils::error)
                    .run_if(resource_changed::<State<SceneState>>()),
            ),
        );
    }
}

fn minimal_scene_setup(mut cmd: Commands) {
    cmd.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: false,
            illuminance: 20000.0,
            ..default()
        },
        transform: Transform::default().looking_at(Vec3::NEG_ONE, Vec3::Y),
        ..default()
    })
    .insert(PrefabMarker);
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
    mut loading_queue: ResMut<LoadingQueue>,
    mut st_next: ResMut<NextState<SceneState>>,
    asset_server: Res<AssetServer>,
) {
    if !loading_queue.queue.is_empty() {
        //filter queue to only include notloaded assets

        let mut not_loaded_count = 0;

        loading_queue.queue.retain(|h| {
            if let Some((load_state, dep_load_state, rec_load_state)) =
                asset_server.get_load_states(h.id())
            {
                let is_loaded = dep_load_state == DependencyLoadState::Loaded;

                let is_failed = dep_load_state == DependencyLoadState::Failed
                    || rec_load_state == RecursiveDependencyLoadState::Failed
                    || load_state == LoadState::Failed;

                if load_state == LoadState::NotLoaded {
                    not_loaded_count += 1;
                }

                !(is_failed || is_loaded)
            } else {
                false
            }
        });

        info!("Loading {} assets", loading_queue.queue.len());

        if loading_queue.queue.len() - not_loaded_count == 0 {
            //not change state
        } else {
            st_next.set(SceneState::Loading);
        }
    } else {
        st_next.set(SceneState::Active);
    }
}

fn auto_add_handle<T: Asset>(
    mut queue: ResMut<LoadingQueue>,
    query: Query<&Handle<T>, Added<Handle<T>>>,
) {
    for h in query.iter() {
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

/// Request the application to load a previously saved `*.scn.ron` dynamic scene
/// file.
///
/// Expects a full, resolvable path to the file (e.g. `scenes/my-scene.scn.ron`
/// for a local file saved under `studio/assets/scenes`, or
/// `remote://<media_token>.scn.ron` for a scene saved to the Storyteller.ai
/// backend).
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
pub struct SceneData {
    pub scene_path: String,
}

#[derive(Component)]
pub struct SceneRoot;

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
}

fn initialiaze_headless(
    mut cmd: Commands,
    mut image_assets: ResMut<Assets<Image>>,
    headless_settings: Res<HeadlessSettings>,
) {
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
        ScreenCapture::from_out(&headless_settings.output_dir),
    ));
}

fn init_object_loading(mut cmd: Commands, mut er: EventReader<LoadLibraryObjectEvent>) {
    if let Some(id) = er.read().last() {
        info!("Loading library asset: {}", id.0);
        cmd.spawn((
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

#[derive(Component)]
pub struct NoMeshElement;

#[derive(Component)]
pub struct InitedKointSkinnedMesh;

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
            Without<NoMeshElement>,
        ),
    >,
    mesh_entities: Query<Entity, With<Handle<Mesh>>>,
) {
    for (e, children) in q_named_non_primitives.iter() {
        let mut has_mesh = false;
        for child in children {
            if mesh_entities.contains(*child) {
                has_mesh = true;
                break;
            }
        }
        if has_mesh {
            cmd.entity(e).insert(SceneElement);
        } else {
            cmd.entity(e).insert(NoMeshElement);
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

fn start_capture(mut captures: Query<&mut ScreenCapture>) {
    for mut capture in captures.iter_mut() {
        capture.start_capturing();
    }
}

#[cfg(feature = "wasm")]
mod wasm {
    use bevy::{prelude::*, window::PrimaryWindow};
    use wasm_bindgen::prelude::*;

    use super::{LoadLibraryObjectEvent, SceneState};
    use crate::wasm::{self, StaticBuffer, WasmMessageBuffer};

    static OBJECT_LOAD_REQUEST: WasmMessageBuffer<String> = WasmMessageBuffer::new();

    #[wasm_bindgen(js_name = loadObject)]
    pub fn load_object(id: &str) {
        OBJECT_LOAD_REQUEST.write_message(id.into());
    }

    // FIXME: Is it an object or a scene? Inconsistent terminology is
    //        unnecessarily confusing
    pub(super) fn dispatch_object_load_request(
        mut ew_load_scene: EventWriter<LoadLibraryObjectEvent>,
    ) {
        if let Some(id) = OBJECT_LOAD_REQUEST.take_message() {
            ew_load_scene.send(LoadLibraryObjectEvent(id));
        }
    }

    #[wasm_bindgen(typescript_custom_section)]
    const SCENE_STATE_EVENT: &str = r#"
export interface SceneStateEvent extends CustomEvent {
    type: "scene-state";
    detail: SceneState;
}

declare global {
    export interface GlobalEventHandlersEventMap {
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
