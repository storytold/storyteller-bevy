use bevy::{
    asset::io::{AssetSource, AssetSourceId},
    prelude::*,
};

use self::reader::RemoteAssetReader;

mod reader;
mod response;

pub struct RemoteAssetPlugin;

impl Plugin for RemoteAssetPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_source(
            AssetSourceId::Name("remote".into()),
            AssetSource::build()
                .with_reader(|| Box::new(RemoteAssetReader))
                .with_writer(|| None),
        );
    }
}
