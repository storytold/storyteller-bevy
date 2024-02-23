use bevy::{gltf::Gltf, prelude::*, scene::InstanceId};

use crate::anim::RetargetSource;

pub struct MixamoPlugin;
impl Plugin for MixamoPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MixamoImportEvent>();
        app.add_state::<MixamoState>();

        app.add_systems(
            Update,
            (
                import_mixamo_scene,
                tick_mixamo_import.run_if(resource_exists::<MixamoImport>()),
                tick_scene_spawning.run_if(in_state(MixamoState::Spawning)),
                start_mixamo_anim.run_if(in_state(MixamoState::Processing)),
            ),
        );
        app.add_systems(OnEnter(MixamoState::Spawning), spawn_mixamo_scene);
    }
}

// TODO: Simplify this flow based on rewin's scene module refactor
#[derive(States, Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub enum MixamoState {
    #[default]
    None,
    Spawning,
    Processing,
}

#[derive(Resource, Deref)]
pub struct MixamoImport(pub Handle<Gltf>);

#[derive(Event, Deref)]
pub struct MixamoImportEvent(pub String);

#[derive(Resource, Deref)]
pub struct MixamoScene(pub InstanceId);

fn import_mixamo_scene(
    mut cmd: Commands,
    r_assets: Res<AssetServer>,
    mut er: EventReader<MixamoImportEvent>,
) {
    if let Some(event) = er.read().last() {
        let path = &**event;
        cmd.insert_resource(MixamoImport(r_assets.load(path)));
    }
}

fn tick_mixamo_import(
    r_mixamo_import: Res<MixamoImport>,
    ra_gltfs: Res<Assets<Gltf>>,
    mut er_scene_events: EventReader<AssetEvent<Scene>>,
    mut st_next: ResMut<NextState<MixamoState>>,
) {
    for event in er_scene_events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            let Some(gltf) = ra_gltfs.get(&**r_mixamo_import) else {
                return;
            };

            if gltf.default_scene.is_some() && gltf.default_scene.as_ref().unwrap().id() == *id {
                st_next.set(MixamoState::Spawning);
                break;
            }
        }
    }
}

fn spawn_mixamo_scene(
    mut cmd: Commands,
    ra_gltfs: Res<Assets<Gltf>>,
    r_mixamo_import: Res<MixamoImport>,
    mut r_scene_spawner: ResMut<SceneSpawner>,
) {
    let gltf = ra_gltfs.get(&**r_mixamo_import).unwrap();
    let scene = gltf.default_scene.as_ref().unwrap();
    let parent = cmd
        .spawn((
            Name::new("Mixamo"),
            SpatialBundle {
                visibility: Visibility::Hidden,
                transform: Transform::from_xyz(0., 0., -1.),
                ..default()
            },
        ))
        .id();

    let inst = r_scene_spawner.spawn_as_child(scene.clone(), parent);
    cmd.insert_resource(MixamoScene(inst));
}

fn tick_scene_spawning(
    r_scene_spawner: Res<SceneSpawner>,
    r_mixamo_scene: Res<MixamoScene>,
    mut st_next: ResMut<NextState<MixamoState>>,
) {
    if r_scene_spawner.instance_is_ready(**r_mixamo_scene) {
        st_next.set(MixamoState::Processing);
    }
}

fn start_mixamo_anim(
    mut cmd: Commands,
    ra_gltfs: Res<Assets<Gltf>>,
    r_mixamo_import: Res<MixamoImport>,
    r_mixamo_scene: Res<MixamoScene>,
    ra_animations: Res<Assets<AnimationClip>>,
    r_scene_spawner: Res<SceneSpawner>,
    q_names: Query<&Name>,
    mut st_next: ResMut<NextState<MixamoState>>,
) {
    let gltf = ra_gltfs.get(&**r_mixamo_import).unwrap();
    let src_anim_handle = gltf.animations[0].clone();
    let src_anim = ra_animations.get(src_anim_handle.clone()).unwrap();

    let Some(src_root) = r_scene_spawner
        .iter_instance_entities(**r_mixamo_scene)
        .find(|ent| match q_names.get(*ent) {
            Ok(name) => src_anim.compatible_with(name),
            Err(_) => false,
        })
    else {
        error!("Failed to find compatible entity for animation");
        return;
    };

    let mut src_player = AnimationPlayer::default();
    src_player.play(src_anim_handle.clone()).repeat();

    cmd.entity(src_root).insert((RetargetSource, src_player));

    st_next.set(MixamoState::None);
}
