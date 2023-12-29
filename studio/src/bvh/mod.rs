use bevy::{prelude::*, ecs::system::{Command, EntityCommands}, asset::AsyncReadExt};
use studio_bvh::joint;

mod error;
mod loader;

use self::loader::BvhLoader;

pub struct BvhPlugin;

impl Plugin for BvhPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Bvh>()
            .register_asset_loader(BvhLoader);
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

#[derive(Component, Debug)]
pub struct BvhAnim {
    pub bvh : Handle<Bvh>,
    pub frame : usize,
}

//==============================================================================
//                        Bvh Anim Component
//==============================================================================

#[derive(Debug, Component)]
pub struct BvhJoint;

//==============================================================================
//                        Commands Plugin
//==============================================================================

pub trait CommandsSpawnBvhExt {
    fn spawn_bvh(&mut self, asset : &Handle<Bvh>) -> Entity;

    fn spawn_bvh_with_transform(&mut self, asset : &Handle<Bvh>, transform : Transform) -> Entity;
}

impl <'w, 's> CommandsSpawnBvhExt for Commands<'w, 's> {
    fn spawn_bvh(&mut self, asset : &Handle<Bvh>) -> Entity {
        let entity = self.spawn_empty().id();
        self.add(SpawnBvhCommand(entity, 1.0, asset.clone()));
        
        self.entity(entity).insert(BvhAnim {
            bvh : asset.clone(),
            frame : 0,
        });
        entity
    }

    fn spawn_bvh_with_transform(&mut self, asset : &Handle<Bvh>, transform : Transform) -> Entity {
        let entity = self.spawn_empty().id();
        self.add(SpawnBvhCommand(entity, 1.0, asset.clone()));
        
        self.entity(entity).insert((
            BvhAnim {
                bvh : asset.clone(),
                frame : 0,
            },
            transform
        ));
        entity
    }
}

pub struct SpawnBvhCommand(Entity, f32, Handle<Bvh>);

impl Command for SpawnBvhCommand {
    fn apply(self, world: &mut World) {
        let bvh = {
            let Some(bvh) = world.get_resource::<Assets<Bvh>>().unwrap().get(self.2)
            else { panic!("Could not create the BVH entity because the bvh file has not been loaded yet") };

            bvh.clone()
        };

        let Some(root) = bvh.0.root_joint()
        else { panic!("Could not create the BVH entity because the bvh file has no root joint") };

        spawn_bvh_recursive(world, &self.0, root);
    }
} 

//==============================================================================
//                        Utility Functions
//==============================================================================

// fn spawn_bvh(commands : &mut Commands, asset : Handle<Bvh>, assets : &Res<Assets<Bvh>>) -> Option<Entity> {
//     let Some(root) = bvh.0.root_joint()
//     else { return None; };

//     let tree = spawn_bvh_recursive(commands, &bvh.0, root);

//     // commands.entity(tree).insert(BvhAnim {
//     //     bvh : bvh.id(),
//     //     frame : 0,
//     // }

//     // Some()
// }

fn spawn_bvh_recursive(world : &mut World, entity : &Entity, joint : studio_bvh::Joint) {
    let offset = joint.data().offset();
    let transform = Transform::from_xyz(offset[0], offset[1], offset[2]);

    // Save the name of the joint
    let name = String::from_utf8_lossy(joint.data().name()).to_string();

    world.entity_mut(*entity).insert((
        SpatialBundle::from_transform(transform), 
        BvhJoint, 
        Name::new(name)
    ));

    for child in joint.children() {
        let mut child_entity = world.spawn_empty().id();
        spawn_bvh_recursive(world, &child_entity, child);
        world.entity_mut(*entity).push_children(&[child_entity]);
    };
}