use bevy::{prelude::*, utils::HashMap};
use space_editor::space_prefab::editor_registry::EditorRegistryExt;

use crate::math::NearlyEq;

use super::{BindPoseGlobal, BindPoseLocal, InvBindPose, SkeletalJoint, Skeleton};

pub(super) struct RetargetingPlugin;
impl Plugin for RetargetingPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SkeletonHeight>()
            .editor_registry::<SkeletonHeight>()
            .register_type::<RetargetMap>();

        app.insert_resource(RetargetMap::default());
        app.add_systems(Update, copy_pose);
    }
}

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct RetargetSource;

#[derive(Component, Clone, Copy, Debug, Default, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct SkeletonHeight(f32);

fn copy_pose(
    mut cmd: Commands,
    r_retarget_map: Res<RetargetMap>,
    q_src_root: Query<Entity, With<RetargetSource>>,
    q_tgt_roots: Query<Entity, (With<Skeleton>, Without<RetargetSource>)>,
    q_parents: Query<&Parent>,
    q_children: Query<&Children>,
    q_joint_names: Query<&Name, With<SkeletalJoint>>,
    q_inv_bind_poses: Query<&InvBindPose>,
    q_world_bind_poses: Query<&BindPoseGlobal>,
    q_local_bind_poses: Query<&BindPoseLocal>,
    q_skel_height: Query<&SkeletonHeight>,
    mut q_xforms: Query<&mut Transform>,
    q_world_xforms: Query<&GlobalTransform>,
) {
    let Ok(src_root) = q_src_root.get_single() else {
        return;
    };

    let src_height = match q_skel_height.get(src_root) {
        Ok(skel_height) => **skel_height,
        Err(_) => {
            let Some(value) = compute_skeleton_height(src_root, &q_children, &q_world_bind_poses)
            else {
                return;
            };
            cmd.entity(src_root).insert(SkeletonHeight(value));

            value
        }
    };

    let src_joints = q_children
        .iter_descendants(src_root)
        .filter_map(|ent| match q_joint_names.get(ent) {
            Ok(name) => Some((ent, name)),
            Err(_) => None,
        })
        .collect::<Vec<_>>();

    // FIXME:
    // This is very much not fast, and probably shouldn't be done every frame.
    // Tentative plan to optimize this without adding too much additional
    // complexity:
    //
    // 1) Add some sort of "AnimationRecorder" component to the target root
    // 2) Beginning with `src_anim.seek_time == 0.0`, record each local
    //    rotation and translation performed on the target bones as a new
    //    keyframe on an `AnimationClip` held by `AnimationRecorder`
    // 3) When `src_anim.seek_time == src_anim.duration`, finalize the new
    //    `AnimationClip` and add it to the asset server
    // 4) Stop running this system and instead start playing the new animation
    //    with an `AnimationPlayer` component on the target root
    for tgt_root in q_tgt_roots.iter() {
        let tgt_height = match q_skel_height.get(tgt_root) {
            Ok(skel_height) => **skel_height,
            Err(_) => {
                let Some(value) =
                    compute_skeleton_height(tgt_root, &q_children, &q_world_bind_poses)
                else {
                    continue;
                };
                cmd.entity(tgt_root).insert(SkeletonHeight(value));

                value
            }
        };
        let xlation_scale = tgt_height / src_height;

        for tgt_joint in q_children.iter_descendants(tgt_root) {
            let Ok(tgt_name) = q_joint_names.get(tgt_joint) else {
                continue;
            };

            // Look up the source bone in the retarget map
            // TODO: Snatching O(n) performance from the jaws of an O(1) data
            //       structure here by using a many-source to single-target
            //       hashmap, but then looking up the source from the target
            //       instead of the other way around
            let Some(src_joint) = src_joints.iter().copied().find_map(|(joint, src_name)| {
                match r_retarget_map.src_to_tgt.get(src_name.as_str()) {
                    Some(name) if name == tgt_name.as_str() => Some(joint),
                    _ => None,
                }
            }) else {
                continue;
            };

            // Get the query data needed to compute the target bone's local rotation
            let Ok(src_inv_bind_pose) = q_inv_bind_poses.get(src_joint) else {
                continue;
            };
            let Ok(src_world_xform) = q_world_xforms.get(src_joint) else {
                continue;
            };
            let Ok(tgt_world_bind_pose) = q_world_bind_poses.get(tgt_joint) else {
                continue;
            };

            // The math here was adapted from this writeup:
            // https://wickedengine.net/2022/09/23/animation-retargeting/
            //
            // ```pseudocode
            // // Apply target-local rotation
            // tgt_working_xform = src_world_xform * src_inv_bind_pose
            // tgt_working_xform = tgt_inv_parent_xform * tgt_working_xform * tgt_world_bind_pose
            // tgt_xform.rotation = tgt_working_xform.rotation
            //
            // // Check if the translation component is animated
            // if src_local_xform.translation != src_local_bind_pose.translation {
            //     // Apply target-local translation
            //     src_world_xlation = src_inv_bind_pose * src_local_xform.translation
            //     tgt_local_xlation = tgt_inv_parent_xform * src_world_xlation
            //     tgt_xform.translation = tgt_local_xlation
            // }
            // ```
            let mut tgt_working_xform = src_world_xform.affine() * (**src_inv_bind_pose);

            // The parent bone's world transform must be computed each iteration,
            // since it will be affected by the rotations applied on the previous
            // iteration.
            let tgt_parent = q_parents.get(tgt_joint);
            let parent_xform = tgt_parent
                .and_then(|parent| q_xforms.get(**parent))
                .copied()
                .unwrap_or_default();

            let mut parent_world_xform = GlobalTransform::from(parent_xform);
            if let Ok(parent) = tgt_parent {
                for ancestor in q_parents.iter_ancestors(**parent) {
                    let xform = q_xforms.get(ancestor).copied().unwrap_or_default();
                    parent_world_xform = xform * parent_world_xform;
                }
            }

            let tgt_inv_parent_xform = parent_world_xform.affine().inverse();
            tgt_working_xform = tgt_inv_parent_xform * tgt_working_xform * (**tgt_world_bind_pose);

            // Apply the local rotation
            if let Ok(mut tgt_xform) = q_xforms.get_mut(tgt_joint) {
                tgt_xform.rotation = Quat::from_affine3(&tgt_working_xform);
            }

            // Get the query data needed to compute the target bone's local translation
            let Ok(src_local_bind_pose) = q_local_bind_poses.get(src_joint) else {
                continue;
            };
            let Ok(src_world_bind_pose) = q_world_bind_poses.get(src_joint) else {
                continue;
            };
            let Ok(src_local_xlation) = q_xforms.get(src_joint).map(|xform| xform.translation)
            else {
                continue;
            };

            // If the source bone translation != its bind pose translation,
            // this translation component is animated (likely root motion)
            if !src_local_xlation.nearly_eq_within(src_local_bind_pose.translation, 1e-3) {
                let src_world_xlation = src_world_bind_pose.transform_vector3(src_local_xlation);
                let tgt_local_xlation = tgt_inv_parent_xform.transform_vector3(src_world_xlation);

                // Apply the local translation
                if let Ok(mut tgt_xform) = q_xforms.get_mut(tgt_joint) {
                    tgt_xform.translation = tgt_local_xlation * xlation_scale;
                }
            }
        }
    }
}

fn compute_skeleton_height(
    root: Entity,
    q_children: &Query<&Children>,
    q_world_bind_poses: &Query<&BindPoseGlobal>,
) -> Option<f32> {
    let joint_positions = q_children
        .iter_descendants(root)
        .filter_map(|joint| {
            q_world_bind_poses
                .get(joint)
                .map(|bind_pose| bind_pose.translation)
                .ok()
        })
        .collect::<Vec<_>>();

    if joint_positions.is_empty() {
        return None;
    }

    let y_max = joint_positions
        .iter()
        .fold(f32::MIN, |accum, current| f32::max(accum, current.y));

    let y_min = joint_positions
        .iter()
        .fold(f32::MAX, |accum, current| f32::min(accum, current.y));

    Some(y_max - y_min)
}

#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource)]
pub struct RetargetMap {
    pub src_to_tgt: HashMap<String, String>,
}

impl RetargetMap {
    const MOCAP_NET: [(&'static str, &'static str); 80] = [
        // Spine
        ("LowerBack", "DEF-spine"),
        ("hip", "DEF-spine"),
        ("abdomen", "DEF-spine.002"),
        ("chest", "DEF-spine.003"),
        ("Neck", "DEF-spine.004"),
        ("neck", "DEF-spine.004"),
        ("Neck1", "DEF-spine.005"),
        ("neck1", "DEF-spine.005"),
        ("Head", "DEF-spine.006"),
        ("head", "DEF-spine.006"),
        // Right arm
        ("rcollar", "DEF-shoulder.R"),
        ("rCollar", "DEF-shoulder.R"),
        ("rshoulder", "DEF-upper_arm.R"),
        ("rShldr", "DEF-upper_arm.R"),
        ("relbow", "DEF-forearm.R"),
        ("rForeArm", "DEF-forearm.R"),
        ("rhand", "DEF-hand.R"),
        ("rHand", "DEF-hand.R"),
        // Right fingers
        // -- Index
        ("metacarpal1.r", "DEF-palm.01.R"),
        ("finger2-1.r", "DEF-f_index.01.R"),
        ("finger2-2.r", "DEF-f_index.02.R"),
        ("finger2-3.r", "DEF-f_index.03.R"),
        // -- Middle
        ("metacarpal2.r", "DEF-palm.02.R"),
        ("finger3-1.r", "DEF-f_middle.01.R"),
        ("finger3-2.r", "DEF-f_middle.02.R"),
        ("finger3-3.r", "DEF-f_middle.03.R"),
        // -- Ring
        ("metacarpal3.r", "DEF-palm.03.R"),
        ("finger4-1.r", "DEF-f_ring.01.R"),
        ("finger4-2.r", "DEF-f_ring.02.R"),
        ("finger4-3.r", "DEF-f_ring.03.R"),
        // -- Pinky
        ("metacarpal4.r", "DEF-palm.04.R"),
        ("finger5-1.r", "DEF-f_pinky.01.R"),
        ("finger5-2.r", "DEF-f_pinky.02.R"),
        ("finger5-3.r", "DEF-f_pinky.03.R"),
        // -- Thumb
        ("rthumb", "DEF-thumb.01.R"),
        ("finger1-2.r", "DEF-thumb.02.R"),
        ("finger1-3.r", "DEF-thumb.03.R"),
        // Right leg
        ("rhip", "DEF-thigh.R"),
        ("rThigh", "DEF-thigh.R"),
        ("rknee", "DEF-shin.R"),
        ("rShin", "DEF-shin.R"),
        ("rfoot", "DEF-foot.R"),
        ("rFoot", "DEF-foot.R"),
        ("RightToeBase", "DEF-toe.R"),
        // Left arm
        ("lcollar", "DEF-shoulder.L"),
        ("lCollar", "DEF-shoulder.L"),
        ("lshoulder", "DEF-upper_arm.L"),
        ("lShldr", "DEF-upper_arm.L"),
        ("lelbow", "DEF-forearm.L"),
        ("lForeArm", "DEF-forearm.L"),
        ("lhand", "DEF-hand.L"),
        ("lHand", "DEF-hand.L"),
        // Left fingers
        // -- Index
        ("metacarpal1.l", "DEF-palm.01.L"),
        ("finger2-1.l", "DEF-f_index.01.L"),
        ("finger2-2.l", "DEF-f_index.02.L"),
        ("finger2-3.l", "DEF-f_index.03.L"),
        // -- Middle
        ("metacarpal2.l", "DEF-palm.02.L"),
        ("finger3-1.l", "DEF-f_middle.01.L"),
        ("finger3-2.l", "DEF-f_middle.02.L"),
        ("finger3-3.l", "DEF-f_middle.03.L"),
        // -- Ring
        ("metacarpal3.l", "DEF-palm.03.L"),
        ("finger4-1.l", "DEF-f_ring.01.L"),
        ("finger4-2.l", "DEF-f_ring.02.L"),
        ("finger4-3.l", "DEF-f_ring.03.L"),
        // -- Pinky
        ("metacarpal4.l", "DEF-palm.04.L"),
        ("finger5-1.l", "DEF-f_pinky.01.L"),
        ("finger5-2.l", "DEF-f_pinky.02.L"),
        ("finger5-3.l", "DEF-f_pinky.03.L"),
        // -- Thumb
        ("lthumb", "DEF-thumb.01.L"),
        ("finger1-2.l", "DEF-thumb.02.L"),
        ("finger1-3.l", "DEF-thumb.03.L"),
        // Left leg
        ("lhip", "DEF-thigh.L"),
        ("lThigh", "DEF-thigh.L"),
        ("lknee", "DEF-shin.L"),
        ("lShin", "DEF-shin.L"),
        ("lfoot", "DEF-foot.L"),
        ("lFoot", "DEF-foot.L"),
        ("LeftToeBase", "DEF-toe.L"),
        // Pelvis
        ("RHipJoint", "DEF-pelvis.R"),
        ("LHipJoint", "DEF-pelvis.L"),
    ];

    const MIXAMO: [(&'static str, &'static str); 60] = [
        // Spine
        ("mixamorig:Hips", "DEF-spine"),
        ("mixamorig:Spine", "DEF-spine.001"),
        ("mixamorig:Spine1", "DEF-spine.002"),
        ("mixamorig:Spine2", "DEF-spine.003"),
        ("mixamorig:Neck", "DEF-spine.004"),
        ("mixamorig:Head", "DEF-spine.006"),
        // Right arm
        ("mixamorig:RightShoulder", "DEF-shoulder.R"),
        ("mixamorig:RightArm", "DEF-upper_arm.R"),
        ("mixamorig:RightForeArm", "DEF-forearm.R"),
        ("mixamorig:RightHand", "DEF-hand.R"),
        // Right fingers
        // -- index
        ("mixamorig:RightHandIndex1", "DEF-palm.01.R"),
        ("mixamorig:RightHandIndex2", "DEF-f_index.01.R"),
        ("mixamorig:RightHandIndex3", "DEF-f_index.02.R"),
        ("mixamorig:RightHandIndex4", "DEF-f_index.03.R"),
        // -- middle
        ("mixamorig:RightHandMiddle1", "DEF-palm.02.R"),
        ("mixamorig:RightHandMiddle2", "DEF-f_middle.01.R"),
        ("mixamorig:RightHandMiddle3", "DEF-f_middle.02.R"),
        ("mixamorig:RightHandMiddle4", "DEF-f_middle.03.R"),
        // -- ring
        ("mixamorig:RightHandRing1", "DEF-palm.03.R"),
        ("mixamorig:RightHandRing2", "DEF-f_ring.01.R"),
        ("mixamorig:RightHandRing3", "DEF-f_ring.02.R"),
        ("mixamorig:RightHandRing4", "DEF-f_ring.03.R"),
        // -- pinky
        ("mixamorig:RightHandPinky1", "DEF-palm.04.R"),
        ("mixamorig:RightHandPinky2", "DEF-f_pinky.01.R"),
        ("mixamorig:RightHandPinky3", "DEF-f_pinky.02.R"),
        ("mixamorig:RightHandPinky4", "DEF-f_pinky.03.R"),
        // -- thumb
        ("mixamorig:RightHandThumb1", "DEF-thumb.01.R"),
        ("mixamorig:RightHandThumb2", "DEF-thumb.02.R"),
        ("mixamorig:RightHandThumb3", "DEF-thumb.03.R"),
        // Right Leg
        ("mixamorig:RightUpLeg", "DEF-thigh.R"),
        ("mixamorig:RightLeg", "DEF-shin.R"),
        ("mixamorig:RightFoot", "DEF-foot.R"),
        ("mixamorig:RightToeBase", "DEF-toe.R"),
        // Left arm
        ("mixamorig:LeftShoulder", "DEF-shoulder.L"),
        ("mixamorig:LeftArm", "DEF-upper_arm.L"),
        ("mixamorig:LeftForeArm", "DEF-forearm.L"),
        ("mixamorig:LeftHand", "DEF-hand.L"),
        // Left fingers
        // -- index
        ("mixamorig:LeftHandIndex1", "DEF-palm.01.L"),
        ("mixamorig:LeftHandIndex2", "DEF-f_index.01.L"),
        ("mixamorig:LeftHandIndex3", "DEF-f_index.02.L"),
        ("mixamorig:LeftHandIndex4", "DEF-f_index.03.L"),
        // -- middle
        ("mixamorig:LeftHandMiddle1", "DEF-palm.02.L"),
        ("mixamorig:LeftHandMiddle2", "DEF-f_middle.01.L"),
        ("mixamorig:LeftHandMiddle3", "DEF-f_middle.02.L"),
        ("mixamorig:LeftHandMiddle4", "DEF-f_middle.03.L"),
        // -- ring
        ("mixamorig:LeftHandRing1", "DEF-palm.03.L"),
        ("mixamorig:LeftHandRing2", "DEF-f_ring.01.L"),
        ("mixamorig:LeftHandRing3", "DEF-f_ring.02.L"),
        ("mixamorig:LeftHandRing4", "DEF-f_ring.03.L"),
        // -- pinky
        ("mixamorig:LeftHandPinky1", "DEF-palm.04.L"),
        ("mixamorig:LeftHandPinky2", "DEF-f_pinky.01.L"),
        ("mixamorig:LeftHandPinky3", "DEF-f_pinky.02.L"),
        ("mixamorig:LeftHandPinky4", "DEF-f_pinky.03.L"),
        // -- thumb
        ("mixamorig:LeftHandThumb1", "DEF-thumb.01.L"),
        ("mixamorig:LeftHandThumb2", "DEF-thumb.02.L"),
        ("mixamorig:LeftHandThumb3", "DEF-thumb.03.L"),
        // Left Leg
        ("mixamorig:LeftUpLeg", "DEF-thigh.L"),
        ("mixamorig:LeftLeg", "DEF-shin.L"),
        ("mixamorig:LeftFoot", "DEF-foot.L"),
        ("mixamorig:LeftToeBase", "DEF-toe.L"),
    ];
}

impl Default for RetargetMap {
    fn default() -> Self {
        Self {
            src_to_tgt: Self::MOCAP_NET
                .iter()
                .chain(Self::MIXAMO.iter())
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }
}
