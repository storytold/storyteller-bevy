use bevy::{prelude::*, utils::HashMap};

use crate::math::NearlyEq;

use super::{BindPoseGlobal, BindPoseLocal, InvBindPose, SkeletalJoint, Skeleton};

pub(super) struct RetargetingPlugin;
impl Plugin for RetargetingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RetargetMap::default());
        app.add_systems(Update, copy_pose);
    }
}

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct RetargetSource;

fn copy_pose(
    r_retarget_map: Res<RetargetMap>,
    q_src_root: Query<Entity, With<RetargetSource>>,
    q_tgt_roots: Query<Entity, (With<Skeleton>, Without<RetargetSource>)>,
    q_parents: Query<&Parent>,
    q_children: Query<&Children>,
    q_joint_names: Query<&Name, With<SkeletalJoint>>,
    q_inv_bind_poses: Query<&InvBindPose>,
    q_world_bind_poses: Query<&BindPoseGlobal>,
    q_local_bind_poses: Query<&BindPoseLocal>,
    mut q_xforms: Query<&mut Transform>,
    q_world_xforms: Query<&GlobalTransform>,
) {
    let src_root = match q_src_root.get_single() {
        Ok(src_root) => src_root,
        Err(err) => {
            error!("{err}");
            return;
        }
    };

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
        for tgt_joint in q_children.iter_descendants(tgt_root) {
            let Ok(tgt_name) = q_joint_names.get(tgt_joint) else {
                continue;
            };

            // Look up the source bone in the retarget map
            let Some(src_joint) =
                r_retarget_map
                    .tgt_to_src
                    .get(tgt_name.as_str())
                    .and_then(|src_name| {
                        q_children.iter_descendants(src_root).find(|ent| {
                            match q_joint_names.get(*ent) {
                                Ok(name) => name.as_str() == src_name,
                                Err(_) => false,
                            }
                        })
                    })
            else {
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
            // tgt_working_xform = tgt_inv_parent_xform * local_xform * tgt_world_bind_pose
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
                // FIXME: This looks really wrong if the source and target
                //        skeletons aren't roughly the same size
                if let Ok(mut tgt_xform) = q_xforms.get_mut(tgt_joint) {
                    tgt_xform.translation = tgt_local_xlation;
                }
            }
        }
    }
}

#[derive(Resource, Debug)]
pub struct RetargetMap {
    pub src_to_tgt: HashMap<String, String>,
    pub tgt_to_src: HashMap<String, String>,
}

impl RetargetMap {
    const MOCAP_NET: [(&str, &str); 80] = [
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
}

impl Default for RetargetMap {
    fn default() -> Self {
        Self {
            src_to_tgt: Self::MOCAP_NET
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            tgt_to_src: Self::MOCAP_NET
                .iter()
                .map(|(v, k)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }
}
