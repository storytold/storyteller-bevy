use bevy::{
    math::Affine3A,
    prelude::*,
    render::{mesh::skinning::SkinnedMesh, view::NoFrustumCulling},
    utils::{EntityHashMap, EntityHashSet},
};
use space_editor::space_prefab::editor_registry::EditorRegistryExt;

use crate::hierarchy;
pub use retargeting::*;

mod retargeting;

pub struct AnimPlugin;
impl Plugin for AnimPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Skeleton>()
            .editor_registry::<Skeleton>()
            .register_type::<SkeletalJoint>()
            .editor_registry::<SkeletalJoint>()
            .register_type::<ProcessedSkinnedMesh>()
            .editor_registry::<ProcessedSkinnedMesh>()
            .register_type::<BindPoseLocal>()
            .editor_registry::<BindPoseLocal>()
            .register_type::<BindPoseGlobal>()
            .editor_registry::<BindPoseGlobal>()
            .register_type::<InvBindPose>()
            .editor_registry::<InvBindPose>();

        app.add_systems(Update, process_skinned_meshes);

        app.add_plugins(retargeting::RetargetingPlugin);
    }
}

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Skeleton;

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct SkeletalJoint(pub Color);

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct ProcessedSkinnedMesh;

#[derive(Component, Clone, Copy, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct BindPoseLocal(pub Transform);

#[derive(Component, Clone, Copy, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct BindPoseGlobal(pub Affine3A);

#[derive(Component, Clone, Copy, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct InvBindPose(pub Affine3A);

fn process_skinned_meshes(
    mut cmd: Commands,
    q_skinned_meshes: Query<(Entity, &SkinnedMesh), Without<ProcessedSkinnedMesh>>,
    mut q_xforms: ParamSet<(TransformHelper, Query<&Transform>)>,
    q_parents: Query<Option<&Parent>>,
) {
    if q_skinned_meshes.is_empty() {
        return;
    }

    // Map of skinned-mesh parent entities to the set of joints bound to those
    // skinned meshes. We use the skinned-mesh parent because individual skinned
    // meshes in glTF format are often only parts of the whole "model," in which
    // case the parts will be siblings in the hierarchy.
    //
    // I.e., considering a hierarchy like this:
    // ```
    // └ RootNode
    //    ├ Body [SkinnedMesh]
    //    ├ Head [SkinnedMesh]
    //    ├ Eyes [SkinnedMesh]
    //    └ Hips [Joint]
    //      ├ UpperLeg_L [Joint]
    //      │ └ ...
    //      ├ UpperLeg_R [Joint]
    //      │ └ ...
    //      └ Spine [Joint]
    //        └ ...
    // ```
    // The hashmap should contain an entry like this:
    // ```
    // {
    //     RootNode: { Hips, UpperLeg_L, UpperLeg_R, Spine, ... }
    // }
    // ```
    // Building this hashmap before processing each joint avoids redundancy in
    // the common case where the same joints are bound to multiple skinned meshes
    let mut joint_collections = EntityHashMap::<Entity, EntityHashSet<Entity>>::default();
    for (ent, skinned_mesh) in q_skinned_meshes.iter() {
        cmd.entity(ent)
            .insert(NoFrustumCulling)
            .insert(ProcessedSkinnedMesh);

        let Some(parent) = q_parents.get(ent).ok().flatten() else {
            continue;
        };
        let parent_ent = **parent;

        let joints = match joint_collections.get_mut(&parent_ent) {
            Some(set) => set,
            None => {
                let (_, set) =
                    joint_collections.insert_unique_unchecked(parent_ent, Default::default());
                set
            }
        };

        joints.extend(skinned_mesh.joints.iter());
    }

    for joints in joint_collections.values() {
        // Find the common ancestor of this joint collection and tag it as the
        // skeleton
        let joints = joints.iter().copied().collect::<Vec<_>>();
        if let Some(root) = hierarchy::nearest_common_ancestor(&joints, &q_parents) {
            cmd.entity(root).insert(Skeleton);
        }

        // Tag each joint and find its bind poses
        for &joint in joints.iter() {
            let bind_pose_global = q_xforms.p0().compute_global_transform(joint).unwrap();
            let bind_pose_local = *q_xforms.p1().get(joint).unwrap();

            cmd.entity(joint).insert((
                SkeletalJoint(Color::BLACK),
                BindPoseLocal(bind_pose_local),
                BindPoseGlobal(bind_pose_global.affine()),
                InvBindPose(bind_pose_global.affine().inverse()),
            ));
        }
    }
}
