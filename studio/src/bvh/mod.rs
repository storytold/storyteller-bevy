use std::backtrace::Backtrace;

use bevy::{
    asset::AsyncReadExt,
    ecs::{
        query::{QueryItem, ReadOnlyWorldQuery, WorldQuery},
        system::{Command, EntityCommands, SystemParam},
    },
    prelude::*,
    transform::commands,
    utils::HashMap,
};
use studio_bvh::{frames::Frame, joint};

mod error;
mod loader;

use crate::scene::{SceneState, SkeletalJoint};

use self::loader::BvhLoader;

pub struct BvhPlugin;

impl Plugin for BvhPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Bvh>()
            .register_asset_loader(BvhLoader)
            .add_systems(
                Update,
                (animate_bvh, retarget_bvh)
                    .chain()
                    .run_if(in_state(SceneState::Active)),
            )
            .register_type::<BvhJoint>()
            .register_type::<BvhAnim>();
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
    #[reflect(ignore)]
    last_frame: usize,
}

pub fn animate_bvh(
    mut command: Commands,
    mut bvh_anims: Query<(Entity, &mut BvhAnim, &mut Transform, &mut Children)>,
    mut bvh_joints: Query<JointQuery, Without<BvhAnim>>,
    bvh_assets: Res<Assets<Bvh>>,
    time: Res<Time>,
) {
    for (entity, mut anim, transform, children) in bvh_anims.iter_mut() {
        let bvh = &bvh_assets
            .get(anim.bvh.clone())
            .expect("Could not animate the BVH because the bvh file has not been loaded yet")
            .0;

        if anim.should_play {
            anim.timer.tick(time.delta());
            if anim.timer.finished() {
                anim.frame = (anim.frame + 1) % bvh.frames().len();
                anim.timer.reset();
                // command.entity(entity).remove::<BvhAnim>();
            }
        }

        if anim.last_frame != anim.frame {
            let frame = bvh
                .frames()
                .nth(anim.frame)
                .expect("Invalid BVH format: the frame index is out of bounds");
            for child in children.iter() {
                apply_frame(&mut bvh_joints, *child, &frame, &bvh);
            }
            anim.last_frame = anim.frame;
        }
    }
}

fn apply_frame<T: ReadOnlyWorldQuery>(
    joint_query: &mut Query<JointQuery, T>,
    current_joint: Entity,
    frame: &Frame,
    bvh_file: &studio_bvh::Bvh,
) {
    let Ok(mut joint_entity) = joint_query.get_mut(current_joint) else {
        return;
    };

    let joint_data = bvh_file
        .get_joint(joint_entity.joint.0)
        .expect("Invalid BVH format: the joint index is out of bounds");

    //new Positions and rotations
    let mut new_pos = joint_entity.transform.translation;
    let mut new_rot: Vec3 = joint_entity
        .transform
        .rotation
        .to_euler(EulerRot::ZXY)
        .into();

    // info!("Joint: {} {} {}", joint_entity.joint.0, String::from_utf8_lossy(joint_data.data().name()), joint_entity.name);

    for channel in joint_data.channels() {
        // info!("Channel {}: {:?} : {}", channel.motion_index(), channel.channel_type(), frame.as_slice()[channel.motion_index()]);
        match channel.channel_type() {
            studio_bvh::ChannelType::PositionX => {
                new_pos.x = frame.as_slice()[channel.motion_index()]
            }
            studio_bvh::ChannelType::PositionY => {
                new_pos.y = frame.as_slice()[channel.motion_index()]
            }
            studio_bvh::ChannelType::PositionZ => {
                new_pos.z = frame.as_slice()[channel.motion_index()]
            }
            studio_bvh::ChannelType::RotationX => {
                new_rot.x = frame.as_slice()[channel.motion_index()]
            }
            studio_bvh::ChannelType::RotationY => {
                new_rot.y = frame.as_slice()[channel.motion_index()]
            }
            studio_bvh::ChannelType::RotationZ => {
                new_rot.z = frame.as_slice()[channel.motion_index()]
            }
        }
    }

    joint_entity.transform.translation = new_pos;
    joint_entity.transform.rotation = Quat::from_euler(
        EulerRot::ZXY,
        new_rot.z.to_radians(),
        new_rot.x.to_radians(),
        new_rot.y.to_radians(),
    );

    if joint_entity.children.is_some() {
        let children = joint_entity
            .children
            .as_mut()
            .unwrap()
            .iter()
            .map(Entity::clone)
            .collect::<Vec<_>>();

        drop(joint_data);
        drop(joint_entity);

        for child in children {
            apply_frame(joint_query, child, frame, bvh_file);
        }
    }
}

fn retarget_bvh(
    bvh_bones: Query<(&Name, &BvhJoint, &Transform), Without<SkeletalJoint>>,
    mut skinned_bones: Query<(&Name, &mut Transform), (Without<BvhJoint>, With<SkeletalJoint>)>,
) {
    let retarget_map = RetargetMap::mocap_to_rigify();
    for (skinned_name, mut skinned_transform) in skinned_bones.iter_mut() {
        for (bvh_name, _, bvh_transform) in bvh_bones.iter() {
            if retarget_map.is_match(&skinned_name, &bvh_name) {
                skinned_transform.rotation = bvh_transform.rotation;
            }
        }
    }
}

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
                    last_frame: 0,
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
        Name::new(name),
    ));

    for child in joint.children() {
        let mut child_entity = world.spawn_empty().id();
        spawn_bvh_recursive(world, &child_entity, child);
        world.entity_mut(*entity).push_children(&[child_entity]);
    }
}

//==============================================================================
//                        RetargetMap
//==============================================================================

#[derive(Debug)]
pub struct RetargetMap {
    pub map: HashMap<String, String>,
}

impl RetargetMap {
    pub fn mocap_to_rigify() -> RetargetMap {
        RetargetMap {
            map: [
                ("DEF-spine", "LowerBack"),
                ("DEF-spine.002", "abdomen"),
                ("DEF-spine.003", "chest"),
                ("DEF-spine.004", "Neck"),
                ("DEF-spine.005", "Neck1"),
                ("DEF-spine.006", "Head"),
                ("DEF-shoulder.R", "rcollar"),
                ("DEF-upper_arm.R", "rshoulder"),
                ("DEF-forearm.R", "relbow"),
                ("DEF-hand.R", "rhand"),
                ("DEF-palm.01.R", "metacarpal1.r"),
                ("DEF-f_index.01.R", "finger2-1.r"),
                ("DEF-f_index.02.R", "finger2-2.r"),
                ("DEF-f_index.03.R", "finger2-3.r"),
                ("DEF-palm.02.R", "metacarpal2.r"),
                ("DEF-f_middle.01.R", "finger3-1.r"),
                ("DEF-f_middle.02.R", "finger3-2.r"),
                ("DEF-f_middle.03.R", "finger3-3.r"),
                ("DEF-palm.03.R", "metacarpal3.r"),
                ("DEF-f_ring.01.R", "finger4-1.r"),
                ("DEF-f_ring.02.R", "finger4-2.r"),
                ("DEF-f_ring.03.R", "finger4-3.r"),
                ("DEF-palm.04.R", "metacarpal4.r"),
                ("DEF-f_pinky.01.R", "finger5-1.r"),
                ("DEF-f_pinky.02.R", "finger5-2.r"),
                ("DEF-f_pinky.03.R", "finger5-3.r"),
                ("DEF-thumb.01.R", "rthumb"),
                ("DEF-thumb.02.R", "finger1-2.r"),
                ("DEF-thumb.03.R", "finger1-3.r"),
                ("DEF-thigh.R", "rhip"),
                ("DEF-shin.R", "rknee"),
                ("DEF-foot.R", "rfoot"),
                ("DEF-toe.R", "RightToeBase"),
                ("DEF-shoulder.L", "lcollar"),
                ("DEF-upper_arm.L", "lshoulder"),
                ("DEF-forearm.L", "lelbow"),
                ("DEF-hand.L", "lhand"),
                ("DEF-palm.01.L", "metacarpal1.l"),
                ("DEF-f_index.01.L", "finger2-1.l"),
                ("DEF-f_index.02.L", "finger2-2.l"),
                ("DEF-f_index.03.L", "finger2-3.l"),
                ("DEF-palm.02.L", "metacarpal2.l"),
                ("DEF-f_middle.01.L", "finger3-1.l"),
                ("DEF-f_middle.02.L", "finger3-2.l"),
                ("DEF-f_middle.03.L", "finger3-3.l"),
                ("DEF-palm.03.L", "metacarpal3.l"),
                ("DEF-f_ring.01.L", "finger4-1.l"),
                ("DEF-f_ring.02.L", "finger4-2.l"),
                ("DEF-f_ring.03.L", "finger4-3.l"),
                ("DEF-palm.04.L", "metacarpal4.l"),
                ("DEF-f_pinky.01.L", "finger5-1.l"),
                ("DEF-f_pinky.02.L", "finger5-2.l"),
                ("DEF-f_pinky.03.L", "finger5-3.l"),
                ("DEF-thumb.01.L", "lthumb"),
                ("DEF-thumb.02.L", "finger1-2.l"),
                ("DEF-thumb.03.L", "finger1-3.l"),
                ("DEF-thigh.L", "lhip"),
                ("DEF-shin.L", "lknee"),
                ("DEF-foot.L", "lfoot"),
                ("DEF-toe.L", "LeftToeBase"),
            ]
            .iter()
            .map(|v| (v.0.to_string(), v.1.to_string()))
            .collect(),
        }
    }

    pub fn is_match(&self, mocapnet_name: &str, rigify_name: &str) -> bool {
        if let Some(translated_name) = self.map.get(mocapnet_name) {
            if rigify_name == translated_name.as_str() {
                return true;
            }
        }

        false
    }
}
