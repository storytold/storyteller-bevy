use bevy::{
    ecs::entity::{EntityHashMap, EntityHashSet},
    math::Affine3A,
    prelude::*,
    render::{mesh::skinning::SkinnedMesh, view::NoFrustumCulling},
    utils::HashMap,
};
use bevy_rapier3d::geometry::Collider;
use space_editor::{prelude::PrefabMarker, space_prefab::editor_registry::EditorRegistryExt};

use crate::{
    hierarchy,
    scene::{SceneElement, SceneState},
};
pub use retargeting::*;

mod retargeting;
pub mod timeline_animation;

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

        app.add_plugins((
            retargeting::RetargetingPlugin,
            timeline_animation::TimelinePlugin,
        ));

        {
            // TODO
            app.register_type::<TempCharacterCollidersProcessed>();
            app.editor_registry::<TempCharacterCollidersProcessed>();

            use ColliderShape::*;

            app.insert_resource(TempCharacterCollidersConfig(
                [
                    (
                        "DEF-spine",
                        vec![TempCharacterColliderConfig {
                            shape: Capsule {
                                radius: 0.108191,
                                length: 0.135549,
                                alignment: BoneAlignment::X,
                            },
                            transform: Transform::from_xyz(0., 0.117, -0.001),
                        }],
                    ),
                    (
                        "DEF-spine.001",
                        vec![TempCharacterColliderConfig {
                            shape: Capsule {
                                radius: 0.091,
                                length: 0.095,
                                alignment: BoneAlignment::X,
                            },
                            transform: Transform::from_xyz(0., 0.068, 0.),
                        }],
                    ),
                    (
                        "DEF-spine.002",
                        vec![
                            TempCharacterColliderConfig {
                                shape: Capsule {
                                    radius: 0.082,
                                    length: 0.062,
                                    alignment: BoneAlignment::X,
                                },
                                transform: Transform::from_xyz(0., 0.006, 0.),
                            },
                            TempCharacterColliderConfig {
                                shape: Capsule {
                                    radius: 0.0823,
                                    length: 0.03661,
                                    alignment: BoneAlignment::X,
                                },
                                transform: Transform::from_xyz(0., 0.088, 0.010),
                            },
                        ],
                    ),
                ]
                .into_iter()
                .collect(),
            ));

            app.add_systems(
                Update,
                temp_attach_colliders_to_character.run_if(in_state(SceneState::Active)),
            );
        }
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
    let mut joint_collections = EntityHashMap::<EntityHashSet>::default();
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
            cmd.entity(root).insert((Skeleton, SceneElement::Skeleton));
        }

        // Tag each joint and find its bind poses
        for &joint in joints.iter() {
            let bind_pose_global = q_xforms.p0().compute_global_transform(joint).unwrap();
            let bind_pose_local = *q_xforms.p1().get(joint).unwrap();

            cmd.entity(joint).insert((
                SkeletalJoint(Color::BLACK),
                SceneElement::Bone,
                BindPoseLocal(bind_pose_local),
                BindPoseGlobal(bind_pose_global.affine()),
                InvBindPose(bind_pose_global.affine().inverse()),
            ));
        }
    }
}

#[derive(Resource, Clone, Default, Deref)]
struct TempCharacterCollidersConfig(HashMap<&'static str, Vec<TempCharacterColliderConfig>>);

#[derive(Debug, Clone, Copy)]
enum BoneAlignment {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy)]
enum ColliderShape {
    Capsule {
        radius: f32,
        length: f32,
        alignment: BoneAlignment,
    },
    Box {
        x: f32,
        y: f32,
        z: f32,
    },
}

#[derive(Debug, Clone, Copy)]
struct TempCharacterColliderConfig {
    shape: ColliderShape,
    transform: Transform,
}

#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
struct TempCharacterCollidersProcessed;

fn temp_attach_colliders_to_character(
    mut cmd: Commands,
    r_colliders_map: Res<TempCharacterCollidersConfig>,
    q_named_prefabs: Query<
        (Entity, &Name),
        (With<PrefabMarker>, Without<TempCharacterCollidersProcessed>),
    >,
    q_named_joints: Query<(Entity, &Name), With<SkeletalJoint>>,
    q_children: Query<&Children>,
) {
    for target in q_named_prefabs.iter().filter_map(|(ent, name)| {
        if name.as_str() == "base-human-female.glb" {
            Some(ent)
        } else {
            None
        }
    }) {
        for (joint, name) in q_children
            .iter_descendants(target)
            .filter_map(|ent| q_named_joints.get(ent).ok())
        {
            if let Some(colliders) = r_colliders_map.get(&name.as_str()) {
                info!("Processing colliders for bone: '{name}'");

                for TempCharacterColliderConfig { shape, transform } in colliders.iter().copied() {
                    cmd.entity(joint).with_children(|cmd| {
                        cmd.spawn((
                            TransformBundle::from_transform(transform),
                            match shape {
                                ColliderShape::Capsule {
                                    radius,
                                    length,
                                    alignment,
                                } => match alignment {
                                    BoneAlignment::X => Collider::capsule_x(length / 2., radius),
                                    BoneAlignment::Y => Collider::capsule_x(length / 2., radius),
                                    BoneAlignment::Z => Collider::capsule_x(length / 2., radius),
                                },
                                ColliderShape::Box { x, y, z } => {
                                    Collider::cuboid(x / 2., y / 2., z / 2.)
                                }
                            },
                            // SceneElement::Collider,
                            bevy_mod_picking::PickableBundle::default(),
                        ));
                    });
                }
            } else {
                warn!("No collider config found for bone: '{name}'");
            }
        }

        cmd.entity(target).insert(TempCharacterCollidersProcessed);
    }
}
