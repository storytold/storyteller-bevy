use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    utils::BoxedFuture,
};

use super::{
    error::{Anyhow, Error},
    Bvh,
};

#[derive(Default)]
pub struct BvhLoader;

impl AssetLoader for BvhLoader {
    type Asset = Bvh;
    type Settings = ();
    type Error = Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _cx: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut buf = vec![];
            reader.read_to_end(&mut buf).await.anyhow()?;
            let bvh = studio_bvh::from_bytes(&buf).anyhow()?;

            Ok(Bvh(bvh))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["bvh"]
    }
}
