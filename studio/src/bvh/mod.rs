use bevy::prelude::*;

mod error;
mod loader;

use self::loader::BvhLoader;

pub struct BvhPlugin;

impl Plugin for BvhPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Bvh>()
            .init_asset::<BvhJoint>()
            .register_asset_loader(BvhLoader);
    }
}

#[derive(Asset, TypePath, Debug)]
pub struct Bvh {
    pub joints: Vec<BvhJoint>,
}

#[derive(Asset, TypePath, Debug, Clone)]
pub struct BvhJoint {
    pub name: String,
    /// Local transform
    pub transform: Transform,
    /// Direct children of this joint
    pub children: Vec<BvhJoint>,
}
