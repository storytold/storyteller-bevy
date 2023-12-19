use crate::{errors::SetMotionError, Channel};
use std::{
    borrow::{Borrow, BorrowMut},
    iter::{DoubleEndedIterator, ExactSizeIterator, FusedIterator, Iterator},
    mem,
    ops::{Index, IndexMut},
    slice::{ChunksExact, ChunksExactMut, SliceIndex},
};

/// An iterator over the frames of a `Bvh`.
///
/// This type is created using the [`Bvh::frames`] method.
///
/// [`Bvh::frames`]: ../struct.Bvh.html#method.frames
#[derive(Debug)]
pub struct Frames<'a> {
    /// Note: `chunks` is wrapped in an option because having a `ChunksExact`
    /// iterator over 0-length chunks panics, and empty `Bvh`s have empty
    /// frames.
    pub(crate) chunks: Option<ChunksExact<'a, f32>>,
}

impl<'a> Iterator for Frames<'a> {
    type Item = Frame<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.chunks.as_mut().and_then(|c| c.next().map(Frame))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.chunks
            .as_ref()
            .map(|c| c.size_hint())
            .unwrap_or((0, Some(0)))
    }
}

impl<'a> DoubleEndedIterator for Frames<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.chunks.as_mut().and_then(|c| c.next_back().map(Frame))
    }
}

impl<'a> ExactSizeIterator for Frames<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.chunks.as_ref().map(|c| c.len()).unwrap_or(0)
    }
}

impl<'a> FusedIterator for Frames<'a> {}

/// A mutable iterator over the frames of a [`Bvh`].
///
/// This type is created using the [`Bvh::frames_mut`] method.
///
/// [`Bvh`]: ../struct.Bvh.html
/// [`Bvh::frames_mut`]: ../struct.Bvh.html#method.frames_mut
#[derive(Debug)]
pub struct FramesMut<'a> {
    /// Note: `chunks` is wrapped in an option for the same reason
    /// that `Frames<'_>`'s `chunks` is wrapped.
    pub(crate) chunks: Option<ChunksExactMut<'a, f32>>,
}

impl<'a> Iterator for FramesMut<'a> {
    type Item = FrameMut<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.chunks.as_mut().and_then(|c| c.next().map(FrameMut))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.chunks
            .as_ref()
            .map(|c| c.size_hint())
            .unwrap_or((0, Some(0)))
    }
}

impl<'a> DoubleEndedIterator for FramesMut<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.chunks
            .as_mut()
            .and_then(|c| c.next_back().map(FrameMut))
    }
}

impl<'a> ExactSizeIterator for FramesMut<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.chunks.as_ref().map(|c| c.len()).unwrap_or(0)
    }
}

impl<'a> FusedIterator for FramesMut<'a> {}

/// A wrapper for a slice of motion values, so that they can be indexed by [`Channel`].
///
/// [`Channel`]: ./struct.Channel.html
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame<'a>(&'a [f32]);

impl<'a> Frame<'a> {
    /// Return the number of values in the `Frame`.
    #[inline]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if there are `0` values in the `FrameMut`. Otherwise,
    /// returns `false`.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Convert the `Frame` into a `&[f32]`.
    #[inline]
    pub const fn as_slice(&self) -> &[f32] {
        self.0
    }

    /// Attempts to get the motion value at `index`. Otherwise, returns `None`.
    #[inline]
    pub fn get<I: FrameIndex>(&self, index: I) -> Option<&Output<I>> {
        self.0.get(index.to_slice_index())
    }
}

impl<'a> AsRef<[f32]> for Frame<'a> {
    #[inline]
    fn as_ref(&self) -> &[f32] {
        self.0
    }
}

impl<'a> Borrow<[f32]> for Frame<'a> {
    #[inline]
    fn borrow(&self) -> &[f32] {
        &*self.0
    }
}

impl<'a, I: FrameIndex> Index<I> for Frame<'a> {
    type Output = Output<I>;
    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        self.0.index(index.to_slice_index())
    }
}

/// A wrapper for a mutable slice of motion values, so that they can be indexed by [`Channel`].
///
/// [`Channel`]: ./struct.Channel.html
#[derive(Debug, PartialEq)]
pub struct FrameMut<'a>(&'a mut [f32]);

impl<'a> FrameMut<'a> {
    /// Return the number of values in the `FrameMut`.
    #[inline]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if there are `0` values in the `FrameMut`. Otherwise,
    /// returns `false`.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Convert the `FrameMut` into a `&[f32]`.
    #[inline]
    pub const fn as_slice(&self) -> &[f32] {
        &*self.0
    }

    /// Convert the `FrameMut` into a `&mut [f32]`.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [f32] {
        self.0
    }

    /// Attempts to get the motion value at `index`. Otherwise, returns `None`.
    #[inline]
    pub fn get<I: FrameIndex>(&self, index: I) -> Option<&Output<I>> {
        self.0.get(index.to_slice_index())
    }

    /// Attempts to return a mutable reference to the motion value at `channel`.
    /// Otherwise, returns `None`.
    #[inline]
    pub fn get_mut<I: FrameIndex>(&'a mut self, index: I) -> Option<&mut Output<I>> {
        self.0.get_mut(index.to_slice_index())
    }

    /// Updates the `motion` value at `channel` to `new_motion`.
    ///
    /// # Notes
    ///
    /// Returns the previous motion value if the operation was successful, and `Err(_)` if
    /// the operation was out of bounds.
    pub fn try_set_motion<I>(&mut self, index: I, new_motion: f32) -> Result<f32, SetMotionError>
    where
        I: FrameIndex<SliceIndex = usize>,
    {
        let index = index.to_slice_index();
        let motion = self
            .0
            .get_mut(index)
            .ok_or(SetMotionError::BadChannel(index))?;
        Ok(mem::replace(motion, new_motion))
    }
}

impl<'a> Borrow<[f32]> for FrameMut<'a> {
    #[inline]
    fn borrow(&self) -> &[f32] {
        &*self.0
    }
}

impl<'a> BorrowMut<[f32]> for FrameMut<'a> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut [f32] {
        self.0
    }
}

impl<'a> AsRef<[f32]> for FrameMut<'a> {
    #[inline]
    fn as_ref(&self) -> &[f32] {
        &*self.0
    }
}

impl<'a> AsMut<[f32]> for FrameMut<'a> {
    #[inline]
    fn as_mut(&mut self) -> &mut [f32] {
        self.0
    }
}

impl<'a, I: FrameIndex> Index<I> for FrameMut<'a> {
    type Output = Output<I>;
    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        self.0.index(index.to_slice_index())
    }
}

impl<'a, I: FrameIndex> IndexMut<I> for FrameMut<'a> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.0.index_mut(index.to_slice_index())
    }
}

impl<'a> From<&'a FrameMut<'a>> for Frame<'a> {
    #[inline]
    fn from(frame_mut: &'a FrameMut<'a>) -> Self {
        Frame(&*frame_mut.0)
    }
}

impl<'a> From<FrameMut<'a>> for Frame<'a> {
    #[inline]
    fn from(frame_mut: FrameMut<'a>) -> Self {
        Frame(&*frame_mut.0)
    }
}

mod private {
    pub trait Sealed {}
}

/// A helper trait used for indexing into a frame.
pub trait FrameIndex: Sized + private::Sealed {
    /// The type of the index generated.
    type SliceIndex: SliceIndex<[f32]>;
    /// Convert `Self` to a `SliceIndex`.
    fn to_slice_index(self) -> Self::SliceIndex;
}

/// Type alias for the result of a `FrameIndex` operation.
pub type Output<I> = <<I as FrameIndex>::SliceIndex as SliceIndex<[f32]>>::Output;

impl<T: SliceIndex<[f32]>> FrameIndex for T {
    type SliceIndex = Self;

    #[inline]
    fn to_slice_index(self) -> Self::SliceIndex {
        self
    }
}

// @TODO: Combine these into one impl for `impl Borrow<Channel>` when specialization lands.
macro_rules! impl_channel_frame_index {
    ($($chn:ty),* $(,)?) => {
        $(
            impl FrameIndex for $chn {
                type SliceIndex = usize;

                #[inline]
                #[inline]
                fn to_slice_index(self) -> Self::SliceIndex {
                    self.motion_index
                }
            }
        )*
    }
}

impl_channel_frame_index!(Channel, &'_ Channel);

impl private::Sealed for Channel {}
impl private::Sealed for &'_ Channel {}
impl<T: SliceIndex<[f32]>> private::Sealed for T {}
