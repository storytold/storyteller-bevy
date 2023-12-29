use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
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

            // FIXME: This is algorithmically terrible
            // let all_joints = bvh.joints().map(|j| j.data().clone()).collect::<Vec<_>>();
            // let joints = bvh
            //     .joints()
            //     .filter_map(|joint| {
            //         if joint.is_root() {
            //             Some(joint.data().clone().into_bvh_joint(&all_joints))
            //         } else {
            //             None
            //         }
            //     })
            //     .collect();

            // bvh.frame_cursor().

            Ok(Bvh(bvh))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["bvh"]
    }
}

// trait IntoBvhJoint: Sized {
//     fn into_bvh_joint(self, data: &[Self]) -> BvhJoint;
// }

// impl IntoBvhJoint for studio_bvh::JointData {
//     fn into_bvh_joint(self, data: &[Self]) -> BvhJoint {
//         BvhJoint {
//             name: String::from_utf8_lossy(self.name()).into(),
//             transform: Transform::from_translation((*self.offset()).into()),
//             children: data
//                 .iter()
//                 .filter_map(|joint| {
//                     joint.parent_index().and_then(|idx| {
//                         if idx == self.index() {
//                             Some(joint.clone().into_bvh_joint(data))
//                         } else {
//                             None
//                         }
//                     })
//                 })
//                 .collect(),
//         }
//     }
// }
