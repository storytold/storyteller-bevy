use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    utils::{BoxedFuture, HashMap},
};

use super::{
    error::{Anyhow, Error},
    Bvh, BvhJoint,
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
        cx: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut buf = vec![];
            reader.read_to_end(&mut buf).await.anyhow()?;

            let bvh = studio_bvh::from_bytes(&buf).anyhow()?;

            let mut joints = Vec::<Handle<BvhJoint>>::new();
            let mut named_joints = HashMap::new();

            for (idx, joint) in bvh.joints().enumerate() {
                let label = format!("Joint{idx}");
                let asset = BvhJoint {
                    transform: Transform::from_translation((*joint.offset()).into()),
                    parent: joint.parent_index().map(|idx| joints[idx].clone()),
                };

                let handle = cx.add_labeled_asset(label, asset);
                joints.push(handle.clone());

                let name = String::from_utf8_lossy(joint.name()).to_string();
                named_joints.insert(name, handle);
            }

            Ok(Bvh {
                joints,
                named_joints,
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["bvh"]
    }
}
