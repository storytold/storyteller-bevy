use bevy::{
    ecs::{
        query::{ReadOnlyWorldQuery, WorldQuery},
        system::Command,
    },
    prelude::*,
    transform::TransformSystem,
};
use studio_bvh::{frames::Frame, ChannelType};

mod error;
mod loader;

use crate::{
    anim::{BindPoseGlobal, BindPoseLocal, InvBindPose, RetargetSource, SkeletalJoint},
    math::Lerp,
    scene::SceneState,
};

use self::loader::BvhLoader;

pub struct BvhPlugin;

impl Plugin for BvhPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Bvh>()
            .add_event::<BvhAnimCompleteEvent>()
            .register_asset_loader(BvhLoader)
            // FIXME: Need to add a state for this module (similar to scene
            //        module's `SceneState`) to manage the order/frequency at
            //        which these systems run instead of just running them all
            //        every frame
            .add_systems(Update, spawn_test_bvh.run_if(resource_exists::<TestBvh>()))
            .add_systems(
                PostUpdate,
                (compute_inverse_bindposes, apply_deferred, animate_bvh)
                    .chain()
                    .run_if(in_state(SceneState::Active))
                    .after(TransformSystem::TransformPropagate),
            )
            .register_type::<BvhJoint>()
            .register_type::<BvhAnim>();
    }
}

// TODO: Rename & reorganize
#[derive(Resource, Deref, DerefMut)]
pub struct TestBvh(pub Handle<Bvh>);

// TODO: Rename & reorganize
fn spawn_test_bvh(
    mut cmd: Commands,
    mut er_bvh_events: EventReader<AssetEvent<Bvh>>,
    r_bvh: Res<TestBvh>,
) {
    for event in er_bvh_events.read() {
        if matches!(event, AssetEvent::LoadedWithDependencies { .. }) {
            let ent = cmd.spawn_bvh_with_transform(
                &r_bvh.0,
                Transform::from_scale(Vec3::splat(0.06))
                    .with_translation(Vec3::new(0., 1.0, 0.524)),
            );

            cmd.entity(ent).insert(RetargetSource);
        }
    }
}

//==============================================================================
//                        Bvh Asset
//==============================================================================

#[derive(Clone, Debug, Default, Asset, TypePath)]
pub struct Bvh(studio_bvh::Bvh);

//==============================================================================
//                        Bvh Anim Component
//==============================================================================

#[derive(Component, Debug, Reflect)]
pub struct BvhAnim {
    #[reflect(ignore)]
    bvh: Handle<Bvh>,
    should_play: bool,
    frame: usize,
    #[reflect(ignore)]
    timer: Timer,
}

// TODO: Rename
fn compute_inverse_bindposes(
    mut cmd: Commands,
    q_bvh_joints: Query<
        (Entity, &Transform, &GlobalTransform),
        (With<BvhJoint>, Without<InvBindPose>),
    >,
) {
    for (joint, xform, world_xform) in q_bvh_joints.iter() {
        cmd.entity(joint).insert((
            BindPoseLocal(*xform),
            BindPoseGlobal(world_xform.affine()),
            InvBindPose(world_xform.affine().inverse()),
        ));
    }
}

pub fn animate_bvh(
    mut q_bvh_anims: Query<(&mut BvhAnim, &Children)>,
    mut q_bvh_joints: Query<JointQuery, Without<BvhAnim>>,
    mut bvh_event_writer: EventWriter<BvhAnimCompleteEvent>,
    r_bvh_assets: Res<Assets<Bvh>>,
    r_time: Res<Time>,
) {
    for (mut anim, children) in q_bvh_anims.iter_mut() {
        let bvh = &r_bvh_assets
            .get(anim.bvh.clone())
            .expect("Could not animate the BVH because the bvh file has not been loaded yet")
            .0;

        let bvh_len = bvh.frames().len();

        let t = if anim.should_play {
            anim.timer.tick(r_time.delta());

            if anim.timer.just_finished() {
                anim.frame = (anim.frame + 1) % bvh_len;
                if anim.frame == 0 {
                    bvh_event_writer.send(BvhAnimCompleteEvent);
                }
            }

            anim.timer.percent()
        } else {
            0.
        };

        let a = bvh.frames().nth(anim.frame).unwrap();
        let b = bvh.frames().nth((anim.frame + 1) % bvh_len).unwrap();

        for &child_ent in children.iter() {
            lerp_frames(&mut q_bvh_joints, child_ent, bvh, &a, &b, t);
        }
    }
}

fn lerp_frames<T: ReadOnlyWorldQuery>(
    joint_query: &mut Query<JointQuery, T>,
    current_joint: Entity,
    bvh: &studio_bvh::Bvh,
    a: &Frame,
    b: &Frame,
    t: f32,
) {
    let Ok(mut joint_entry) = joint_query.get_mut(current_joint) else {
        return;
    };

    let joint_data = bvh
        .get_joint(joint_entry.joint.0)
        .expect("Invalid BVH format: the joint index is out of bounds");

    //new Positions and rotations
    let mut new_pos = joint_entry.transform.translation;
    let mut new_rot: Vec3 = joint_entry
        .transform
        .rotation
        .to_euler(EulerRot::ZXY)
        .into();

    // info!("Joint: {} {} {}", joint_entity.joint.0, String::from_utf8_lossy(joint_data.data().name()), joint_entity.name);

    for channel in joint_data.channels() {
        // info!("Channel {}: {:?} : {}", channel.motion_index(), channel.channel_type(), frame.as_slice()[channel.motion_index()]);
        let target = match channel.channel_type() {
            ChannelType::PositionX => &mut new_pos.x,
            ChannelType::PositionY => &mut new_pos.y,
            ChannelType::PositionZ => &mut new_pos.z,
            ChannelType::RotationX => &mut new_rot.x,
            ChannelType::RotationY => &mut new_rot.y,
            ChannelType::RotationZ => &mut new_rot.z,
        };

        *target = t.lerp(
            a.as_slice()[channel.motion_index()],
            b.as_slice()[channel.motion_index()],
        );
    }

    joint_entry.transform.translation = new_pos;
    joint_entry.transform.rotation = Quat::from_euler(
        // TODO: Read the Euler order from the BVH instead of hard-coding
        EulerRot::ZXY,
        new_rot.z.to_radians(),
        new_rot.x.to_radians(),
        new_rot.y.to_radians(),
    );

    if joint_entry.children.is_some() {
        let children = joint_entry
            .children
            .unwrap()
            .iter()
            .copied()
            .collect::<Vec<_>>();

        for child in children {
            lerp_frames(joint_query, child, bvh, a, b, t);
        }
    }
}

//==============================================================================
//                        Bvh Anim Events
//==============================================================================

// This may seem weird, but this is for the Headless mode to
// know when to stop recording.

#[derive(Event)]
pub struct BvhAnimCompleteEvent;

//==============================================================================
//                        Bvh Anim Component
//==============================================================================

#[derive(Debug, Component, Reflect)]
pub struct BvhJoint(usize);

#[derive(Debug, WorldQuery)]
#[world_query(mutable)]
pub struct JointQuery {
    pub name: &'static Name,
    pub joint: &'static BvhJoint,
    pub transform: &'static mut Transform,
    pub children: Option<&'static mut Children>,
}

//==============================================================================
//                        Commands Plugin
//==============================================================================

pub trait CommandsSpawnBvhExt {
    fn spawn_bvh(&mut self, asset: &Handle<Bvh>) -> Entity;

    fn spawn_bvh_with_transform(&mut self, asset: &Handle<Bvh>, transform: Transform) -> Entity;
}

impl<'w, 's> CommandsSpawnBvhExt for Commands<'w, 's> {
    fn spawn_bvh(&mut self, asset: &Handle<Bvh>) -> Entity {
        let entity = self.spawn_empty().id();
        self.add(SpawnBvhCommand(entity, asset.clone(), Transform::default()));
        entity
    }

    fn spawn_bvh_with_transform(&mut self, asset: &Handle<Bvh>, transform: Transform) -> Entity {
        let entity = self.spawn_empty().id();
        self.add(SpawnBvhCommand(entity, asset.clone(), transform));
        entity
    }
}

pub struct SpawnBvhCommand(Entity, Handle<Bvh>, Transform);

impl Command for SpawnBvhCommand {
    fn apply(self, world: &mut World) {
        let bvh = {
            let Some(bvh) = world.get_resource::<Assets<Bvh>>().unwrap().get(&self.1) else {
                panic!(
                    "Could not create the BVH entity because the bvh file has not been loaded yet"
                )
            };

            bvh.clone()
        };

        let Some(root) = bvh.0.root_joint() else {
            panic!("Could not create the BVH entity because the bvh file has no root joint")
        };

        let entity = world.spawn_empty().id();
        spawn_bvh_recursive(world, &entity, root);
        world
            .entity_mut(self.0)
            .insert((
                BvhAnim {
                    bvh: self.1,
                    frame: 0,
                    should_play: true,
                    timer: Timer::from_seconds(
                        bvh.0.frame_time().as_secs_f32(),
                        TimerMode::Repeating,
                    ),
                },
                SpatialBundle::from_transform(self.2),
                Name::new("BVH Skeliton"),
            ))
            .add_child(entity);
    }
}

//==============================================================================
//                        Utility Functions
//==============================================================================

fn spawn_bvh_recursive(world: &mut World, entity: &Entity, joint: studio_bvh::Joint) {
    let offset = joint.data().offset();
    let transform = Transform::from_xyz(offset[0], offset[1], offset[2]);

    // Save the name of the joint
    let name = String::from_utf8_lossy(joint.data().name()).to_string();

    world.entity_mut(*entity).insert((
        SpatialBundle::from_transform(transform),
        BvhJoint(joint.data().index()),
        SkeletalJoint(Color::WHITE),
        Name::new(name),
    ));

    for child in joint.children() {
        let child_entity = world.spawn_empty().id();
        spawn_bvh_recursive(world, &child_entity, child);
        world.entity_mut(*entity).push_children(&[child_entity]);
    }
}
