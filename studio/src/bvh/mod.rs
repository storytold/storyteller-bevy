use bevy::{
    ecs::{
        query::{ReadOnlyWorldQuery, WorldQuery},
        system::Command,
    },
    prelude::*,
    transform::TransformSystem,
    utils::HashMap,
};
use studio_bvh::{frames::Frame, ChannelType};

mod error;
mod loader;

use crate::{
    math::Lerp,
    scene::{BindPoseGlobal, InvBindPose, SceneState, SkeletalJoint},
};

use self::loader::BvhLoader;

pub struct BvhPlugin;

impl Plugin for BvhPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Bvh>()
            .register_asset_loader(BvhLoader)
            // FIXME: Need to add a state for this module (similar to scene
            //        module's `SceneState`) to manage the order/frequency at
            //        which these systems run instead of just running them all
            //        every frame
            .add_systems(
                PostUpdate,
                (
                    compute_inverse_bindposes,
                    apply_deferred,
                    animate_bvh,
                    retarget_bvh,
                )
                    .chain()
                    .run_if(in_state(SceneState::Active))
                    .after(TransformSystem::TransformPropagate),
            )
            .register_type::<BvhJoint>()
            .register_type::<BvhAnim>()
            .insert_resource(RetargetMap::default());
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

fn compute_inverse_bindposes(
    mut cmd: Commands,
    q_bvh_joints: Query<(Entity, &GlobalTransform), (With<BvhJoint>, Without<InvBindPose>)>,
) {
    for (ent, world_xform) in q_bvh_joints.iter() {
        let inv_bind_pose = world_xform.compute_matrix().inverse();
        cmd.entity(ent).insert(InvBindPose(inv_bind_pose));
    }
}

pub fn animate_bvh(
    mut q_bvh_anims: Query<(&mut BvhAnim, &Children)>,
    mut q_bvh_joints: Query<JointQuery, Without<BvhAnim>>,
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
                anim.timer.reset();
                0.
            } else {
                anim.timer.percent()
            }
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

fn retarget_bvh(
    r_retarget_map: Res<RetargetMap>,
    mut l_target_ents: Local<HashMap<String, Entity>>,
    q_source_bones: Query<(&Name, &InvBindPose, &GlobalTransform), With<BvhJoint>>,
    q_target_ents: Query<(Entity, &Name), (With<SkeletalJoint>, Without<BvhJoint>)>,
    mut q_target_xforms: Query<
        (&Parent, &BindPoseGlobal, &mut Transform),
        (With<SkeletalJoint>, Without<BvhJoint>),
    >,
    q_world_xforms: Query<&GlobalTransform>,
) {
    // let retarget_map = RetargetMap::mocap_to_rigify();
    for (source_name, source_inv_bind_pose, source_world_xform) in q_source_bones.iter() {
        // Lookup the target name in the retarget map
        let Some(target_name) = r_retarget_map.source_to_target.get(source_name.as_str()) else {
            continue;
        };

        // Lookup the target entity in the local target bones hashmap,
        // or insert it if this is the system's first run
        let target_ent = match l_target_ents.get(target_name) {
            Some(ent) => Some(*ent),
            None => {
                if let Some(ent) = q_target_ents.iter().find_map(|(ent, name)| {
                    if name.as_str() == target_name {
                        Some(ent)
                    } else {
                        None
                    }
                }) {
                    l_target_ents.insert(target_name.clone(), ent);
                    Some(ent)
                } else {
                    None
                }
            }
        };

        // Lookup the transform data for the matching entity
        let Some((target_parent_ent, target_bind_pose_global, mut target_xform)) =
            target_ent.and_then(|ent| q_target_xforms.get_mut(ent).ok())
        else {
            continue;
        };

        // Copy rotations from source -> target
        //
        // This code is adapted from the writeup here:
        //   https://wickedengine.net/2022/09/23/animation-retargeting/
        // with the main difference being that the order of operations is
        // exactly reversed (DirectX uses row-major matrices, while Bevy uses
        // column-major ordering)

        // FIXME: We're seeing some jitteryness due to numerical instability.
        //        Should refactor the Mat4's to Affine3A's and periodically re-
        //        orthonormalize the matrices inbetween operations.

        // [psuedo] local_mat = source_inv_bind_mat * source_world_mat;
        let mut local_mat = source_world_xform.compute_matrix() * (**source_inv_bind_pose);

        // [psuedo] local_mat = target_world_mat * local_mat * target_inv_parent_mat;
        let parent_world_xform = match q_world_xforms.get(**target_parent_ent) {
            Ok(world_xform) => world_xform,
            Err(_) => &GlobalTransform::IDENTITY,
        };
        let target_inv_parent_mat = parent_world_xform.compute_matrix().inverse();
        local_mat = target_inv_parent_mat * local_mat * (**target_bind_pose_global);

        // [psuedo] target_xform.rotation = local_mat.rotation;
        let (_, rotation, _) = local_mat.to_scale_rotation_translation();
        target_xform.rotation = rotation;
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

//==============================================================================
//                        RetargetMap
//==============================================================================

#[derive(Resource, Debug)]
pub struct RetargetMap {
    pub target_to_source: HashMap<String, String>,
    pub source_to_target: HashMap<String, String>,
}

#[rustfmt::skip]
static TARGET_TO_SOURCE: [(&str, &str); 62] = [
    // Spine
    ("DEF-spine", "LowerBack"),
    ("DEF-spine.002", "abdomen"),
    ("DEF-spine.003", "chest"),
    ("DEF-spine.004", "Neck"),
    ("DEF-spine.005", "Neck1"),
    ("DEF-spine.006", "Head"),

    // Right arm
    ("DEF-shoulder.R", "rcollar"),
    ("DEF-upper_arm.R", "rshoulder"),
    ("DEF-forearm.R", "relbow"),
    ("DEF-hand.R", "rhand"),

    // Right fingers
    // -- Index
    ("DEF-palm.01.R", "metacarpal1.r"),
    ("DEF-f_index.01.R", "finger2-1.r"),
    ("DEF-f_index.02.R", "finger2-2.r"),
    ("DEF-f_index.03.R", "finger2-3.r"),
    // -- Middle
    ("DEF-palm.02.R", "metacarpal2.r"),
    ("DEF-f_middle.01.R", "finger3-1.r"),
    ("DEF-f_middle.02.R", "finger3-2.r"),
    ("DEF-f_middle.03.R", "finger3-3.r"),
    // -- Ring
    ("DEF-palm.03.R", "metacarpal3.r"),
    ("DEF-f_ring.01.R", "finger4-1.r"),
    ("DEF-f_ring.02.R", "finger4-2.r"),
    ("DEF-f_ring.03.R", "finger4-3.r"),
    // -- Pinky
    ("DEF-palm.04.R", "metacarpal4.r"),
    ("DEF-f_pinky.01.R", "finger5-1.r"),
    ("DEF-f_pinky.02.R", "finger5-2.r"),
    ("DEF-f_pinky.03.R", "finger5-3.r"),
    // -- Thumb
    ("DEF-thumb.01.R", "rthumb"),
    ("DEF-thumb.02.R", "finger1-2.r"),
    ("DEF-thumb.03.R", "finger1-3.r"),

    // Right leg
    ("DEF-thigh.R", "rhip"),
    ("DEF-shin.R", "rknee"),
    ("DEF-foot.R", "rfoot"),
    ("DEF-toe.R", "RightToeBase"),

    // Left arm
    ("DEF-shoulder.L", "lcollar"),
    ("DEF-upper_arm.L", "lshoulder"),
    ("DEF-forearm.L", "lelbow"),
    ("DEF-hand.L", "lhand"),

    // Left fingers
    // -- Index
    ("DEF-palm.01.L", "metacarpal1.l"),
    ("DEF-f_index.01.L", "finger2-1.l"),
    ("DEF-f_index.02.L", "finger2-2.l"),
    ("DEF-f_index.03.L", "finger2-3.l"),

    // -- Middle
    ("DEF-palm.02.L", "metacarpal2.l"),
    ("DEF-f_middle.01.L", "finger3-1.l"),
    ("DEF-f_middle.02.L", "finger3-2.l"),
    ("DEF-f_middle.03.L", "finger3-3.l"),

    // -- Ring
    ("DEF-palm.03.L", "metacarpal3.l"),
    ("DEF-f_ring.01.L", "finger4-1.l"),
    ("DEF-f_ring.02.L", "finger4-2.l"),
    ("DEF-f_ring.03.L", "finger4-3.l"),

    // -- Pinky
    ("DEF-palm.04.L", "metacarpal4.l"),
    ("DEF-f_pinky.01.L", "finger5-1.l"),
    ("DEF-f_pinky.02.L", "finger5-2.l"),
    ("DEF-f_pinky.03.L", "finger5-3.l"),

    // -- Thumb
    ("DEF-thumb.01.L", "lthumb"),
    ("DEF-thumb.02.L", "finger1-2.l"),
    ("DEF-thumb.03.L", "finger1-3.l"),

    // Left leg
    ("DEF-thigh.L", "lhip"),
    ("DEF-shin.L", "lknee"),
    ("DEF-foot.L", "lfoot"),
    ("DEF-toe.L", "LeftToeBase"),

    // Pelvis
    ("DEF-pelvis.R", "RHipJoint"),
    ("DEF-pelvis.L", "LHipJoint"),
];

impl Default for RetargetMap {
    fn default() -> Self {
        Self {
            target_to_source: TARGET_TO_SOURCE
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            source_to_target: TARGET_TO_SOURCE
                .iter()
                .map(|(v, k)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }
}
