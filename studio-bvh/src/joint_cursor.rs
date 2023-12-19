use crate::Bvh;
use mint::Vector3;

#[derive(Debug)]
pub struct JointCursor<'a> {
    bvh: &'a mut Bvh,
    index: usize,
}

impl<'a> JointCursor<'a> {
    #[inline]
    pub const fn index(&self) -> usize {
        self.index
    }

    #[inline]
    pub fn push_joints(&mut self) -> PushJointsChain<'a> {
        PushJointsChain { cursor: self }
    }
}

#[derive(Debug)]
pub struct PushJointsChain<'a> {
    cursor: &'a mut JointCursor<'a>,
}

impl<'a> PushJointsChain<'a> {
    pub fn push_joint<B: AsRef<[u8]>, V: Into<Vector3<f32>>>(
        &mut self,
        name: B,
        offset: V,
    ) -> Self {
        todo!()
    }
}

impl<'a> From<&'a mut Bvh> for JointCursor<'a> {
    #[inline]
    fn from(bvh: &'a mut Bvh) -> Self {
        Self { bvh, index: 0 }
    }
}
