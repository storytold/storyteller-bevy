use crate::{
    errors::{FrameInsertError, FrameRemoveError},
    frames::{Frame, FrameMut, Frames, FramesMut},
    Bvh,
};
use std::cmp::min;

/// A `FrameCursor` is used to insert frames into a [`Bvh`]. It operates
/// similarly to a [`linked_list::Cursor`]: methods are provided to move
/// through frames, as well as insert and delete frames.
///
/// You can create a `FrameCursor` using the [`Bvh::frame_cursor`] method.
///
/// [`Bvh`]: ../struct.Bvh.html
/// [`linked_list::Cursor`]: http://doc.rust-lang.org/stable/std/collections/linked_list/struct.Cursor.html
/// [`Bvh::frame_cursor`]: ../struct.Bvh.html#method.frame_cursor
#[derive(Debug)]
pub struct FrameCursor<'bvh> {
    bvh: &'bvh mut Bvh,
    index: usize,
}

impl<'bvh> FrameCursor<'bvh> {
    /// Returns the current index of the `FrameCursor`.
    #[inline]
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Returns the number of frames in the [`Bvh`] the cursor is is currently
    /// pointing to.
    ///
    /// [`Bvh`]: ../struct.Bvh.html
    #[inline]
    pub fn len(&self) -> usize {
        self.bvh.frames().len()
    }

    /// Returns the number of channels in the [`Bvh`] the cursor is is currently
    /// pointing to.
    ///
    /// [`Bvh`]: ../struct.Bvh.html
    #[inline]
    pub const fn num_channels(&self) -> usize {
        self.bvh.num_channels
    }

    /// Move the `FrameCursor` to the next frame of the `Bvh`'s motion values.
    ///
    /// If the `FrameCursor` is currently at the last frame, then this method will
    /// do nothing.
    #[inline]
    pub fn move_next(&mut self) -> &mut Self {
        self.index = min(self.index + 1, self.bvh.frames().len());
        self
    }

    /// Move the `FrameCursor` to the previous frame of the `Bvh`'s motion values.
    ///
    /// If the `FrameCursor` is currently at the first frame, then this method will
    /// do nothing.
    #[inline]
    pub fn move_prev(&mut self) -> &mut Self {
        self.index = self.index.saturating_sub(1);
        self
    }

    /// Move the `FrameCursor` to the first frame of the `Bvh`'s motion values.
    ///
    /// If the `FrameCursor` is currently at the first frame, then this method will
    /// do nothing.
    #[inline]
    pub fn move_first(&mut self) -> &mut Self {
        self.index = 0;
        self
    }

    /// Move the `FrameCursor` to the last frame of the `Bvh`'s motion values.
    ///
    /// If the `FrameCursor` is currently at the last frame, then this method will
    /// do nothing.
    #[inline]
    pub fn move_last(&mut self) -> &mut Self {
        self.index = self.len();
        self
    }

    /// Returns the frame next to the current index of the `FrameCursor`.
    ///
    /// Returns `None` if there is no frame available.
    #[inline]
    pub fn peek_next(&self) -> Option<Frame<'_>> {
        self.bvh.frames().nth(self.index)
    }

    /// Returns the frame before the current index of the `FrameCursor`.
    ///
    /// Returns `None` if there is no frame available.
    #[inline]
    pub fn peek_prev(&self) -> Option<Frame<'_>> {
        if self.index() > 0 {
            self.bvh.frames().nth(self.index - 1)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the frame next to the current
    /// index of the `FrameCursor`.
    ///
    /// Returns `None` if there is no frame available.
    #[inline]
    pub fn peek_next_mut(&mut self) -> Option<FrameMut<'_>> {
        self.bvh.frames_mut().nth(self.index)
    }

    /// Returns a mutable reference to the frame before the current index
    /// of the `FrameCursor`.
    ///
    /// Returns `None` if there is no frame available.
    #[inline]
    pub fn peek_prev_mut(&mut self) -> Option<FrameMut<'_>> {
        if self.index() > 0 {
            self.bvh.frames_mut().nth(self.index - 1)
        } else {
            None
        }
    }

    /// Returns the frames surrounding the current index of the `FrameCursor`.
    ///
    /// If either of the frames is not available, then `None` will be returned
    /// for that frame.
    #[inline]
    pub fn surrounding_frames(&self) -> (Option<Frame<'_>>, Option<Frame<'_>>) {
        (self.peek_prev(), self.peek_next())
    }

    /// Attempt to insert a new frame of animation after the current index.
    /// The index of the cursor will be advanced to the end of the frames inserted.
    ///
    /// # Errors
    ///
    /// If the frame could not be inserted (because it was an incorrect size),
    /// then an error will be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() {
    /// # use bvh_anim::bvh;
    /// let mut simple_skeleton = bvh! {
    ///     HIERARCHY
    ///     ROOT Base
    ///     {
    ///         // Hierarchy omitted...
    ///     #     OFFSET 0.0 0.0 0.0
    ///     #     CHANNELS 6 Xposition Yposition Zposition Zrotation Xrotation Yrotation
    ///     #     JOINT Middle1
    ///     #     {
    ///     #         OFFSET 0.0 0.0 15.0
    ///     #         CHANNELS 3 Zrotation Xrotation Yrotation
    ///     #         JOINT Tip1
    ///     #         {
    ///     #             OFFSET 0.0 0.0 30.0
    ///     #             CHANNELS 3 Zrotation Xrotation Yrotation
    ///     #             End Site
    ///     #             {
    ///     #                 OFFSET 0.0 0.0 45.0
    ///     #             }
    ///     #         }
    ///     #     }
    ///     #     JOINT Middle2
    ///     #     {
    ///     #         OFFSET 0.0 15.0 0.0
    ///     #         CHANNELS 3 Zrotation Xrotation Yrotation
    ///     #         JOINT Tip2
    ///     #         {
    ///     #             OFFSET 0.0 30.0 0.0
    ///     #             CHANNELS 3 Zrotation Xrotation Yrotation
    ///     #             End Site
    ///     #             {
    ///     #                 OFFSET 0.0 45.0 0.0
    ///     #             }
    ///     #         }
    ///     #     }
    ///     }
    ///
    ///     MOTION
    ///     Frames: 3
    ///     Frame Time: 0.033333333333
    ///     0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0
    ///     1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0
    ///     2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0
    /// };
    ///
    /// let mut frame_cursor = simple_skeleton.frame_cursor();
    /// frame_cursor.move_last();
    ///
    /// assert_eq!(frame_cursor.index(), 3);
    ///
    /// frame_cursor
    ///     .try_insert_frame(&[3.0; 18])
    ///     .expect("Could not insert frame");
    ///
    /// assert_eq!(frame_cursor.index(), 4);
    /// # drop(frame_cursor);
    /// assert_eq!(simple_skeleton.frames().len(), 4);
    ///
    /// # for (i, frame) in simple_skeleton.frames().enumerate() {
    /// #     for motion in &frame.as_slice()[..] {
    /// #         assert_eq!(*motion, i as f32);
    /// #     }
    /// # }
    /// # } // fn main()
    /// ```
    pub fn try_insert_frame<F>(&mut self, frame: &F) -> Result<&mut Self, FrameInsertError>
    where
        F: AsRef<[f32]> + ?Sized,
    {
        let frame = frame.as_ref();
        if frame.len() != self.num_channels() {
            return Err(FrameInsertError::incorrect_len(
                self.num_channels(),
                frame.len(),
            ));
        }

        let idx = self.index * self.num_channels();
        vec_insert_slice(&mut self.bvh.motion_values, idx, frame);

        Ok(self.move_next())
    }

    /// Insert multiple frames contiguously at the current index. This is generally more
    /// efficient than calling [`try_insert_frame`] multiple times because this method will
    /// preallocate space for the multiple frames.
    ///
    /// The index of the cursor will be advanced to the end of the frames inserted.
    ///
    /// # Errors
    ///
    /// If any of the frames could not be inserted (because they were an incorrect size),
    /// then an error will be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() {
    /// # use bvh_anim::bvh;
    /// let mut simple_skeleton = bvh! {
    ///     HIERARCHY
    ///     ROOT Base
    ///     {
    ///         // Hierarchy omitted...
    ///     #     OFFSET 0.0 0.0 0.0
    ///     #     CHANNELS 6 Xposition Yposition Zposition Zrotation Xrotation Yrotation
    ///     #     JOINT Middle1
    ///     #     {
    ///     #         OFFSET 0.0 0.0 15.0
    ///     #         CHANNELS 3 Zrotation Xrotation Yrotation
    ///     #         JOINT Tip1
    ///     #         {
    ///     #             OFFSET 0.0 0.0 30.0
    ///     #             CHANNELS 3 Zrotation Xrotation Yrotation
    ///     #             End Site
    ///     #             {
    ///     #                 OFFSET 0.0 0.0 45.0
    ///     #             }
    ///     #         }
    ///     #     }
    ///     #     JOINT Middle2
    ///     #     {
    ///     #         OFFSET 0.0 15.0 0.0
    ///     #         CHANNELS 3 Zrotation Xrotation Yrotation
    ///     #         JOINT Tip2
    ///     #         {
    ///     #             OFFSET 0.0 30.0 0.0
    ///     #             CHANNELS 3 Zrotation Xrotation Yrotation
    ///     #             End Site
    ///     #             {
    ///     #                 OFFSET 0.0 45.0 0.0
    ///     #             }
    ///     #         }
    ///     #     }
    ///     }
    ///
    ///     MOTION
    ///     Frames: 3
    ///     Frame Time: 0.033333333333
    ///     0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0
    ///     1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0
    ///     2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0
    /// };
    ///
    /// simple_skeleton
    ///     .frame_cursor()
    ///     .move_last()
    ///     .try_insert_frames(&[[3.0; 18], [4.0; 18]])
    ///     .expect("Could not insert frames");
    ///
    /// assert_eq!(simple_skeleton.frames().len(), 5);
    ///
    /// # for (i, frame) in simple_skeleton.frames().enumerate() {
    /// #     for motion in &frame.as_slice()[..] {
    /// #         assert_eq!(*motion, i as f32);
    /// #     }
    /// # }
    /// # } // fn main()
    /// ```
    ///
    ///[`try_insert_frame`]: ./struct.FrameInserter.html#method.try_insert_frame
    pub fn try_insert_frames<'f, I, F>(&mut self, frames: I) -> Result<&mut Self, FrameInsertError>
    where
        I: IntoIterator<Item = &'f F>,
        F: 'f + AsRef<[f32]> + ?Sized,
    {
        let frames = frames.into_iter();
        let num_channels = self.num_channels();
        let num_frames = {
            let (low, up) = frames.size_hint();
            up.unwrap_or(low)
        };

        self.bvh.motion_values.reserve(num_frames * num_channels);

        for frame in frames.into_iter() {
            let frame = frame.as_ref();
            self.try_insert_frame(&frame)?;
        }

        Ok(self)
    }

    /// Attempt to remove a frame of animation at the current index.
    ///
    /// # Errors
    ///
    /// If the frame could not be removed, then an error will be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() {
    /// # use bvh_anim::bvh;
    /// let mut simple_skeleton = bvh! {
    ///     HIERARCHY
    ///     ROOT Base
    ///     {
    ///         // Hierarchy omitted...
    ///     #     OFFSET 0.0 0.0 0.0
    ///     #     CHANNELS 6 Xposition Yposition Zposition Zrotation Xrotation Yrotation
    ///     #     JOINT Middle1
    ///     #     {
    ///     #         OFFSET 0.0 0.0 15.0
    ///     #         CHANNELS 3 Zrotation Xrotation Yrotation
    ///     #         JOINT Tip1
    ///     #         {
    ///     #             OFFSET 0.0 0.0 30.0
    ///     #             CHANNELS 3 Zrotation Xrotation Yrotation
    ///     #             End Site
    ///     #             {
    ///     #                 OFFSET 0.0 0.0 45.0
    ///     #             }
    ///     #         }
    ///     #     }
    ///     #     JOINT Middle2
    ///     #     {
    ///     #         OFFSET 0.0 15.0 0.0
    ///     #         CHANNELS 3 Zrotation Xrotation Yrotation
    ///     #         JOINT Tip2
    ///     #         {
    ///     #             OFFSET 0.0 30.0 0.0
    ///     #             CHANNELS 3 Zrotation Xrotation Yrotation
    ///     #             End Site
    ///     #             {
    ///     #                 OFFSET 0.0 45.0 0.0
    ///     #             }
    ///     #         }
    ///     #     }
    ///     }
    ///
    ///     MOTION
    ///     Frames: 3
    ///     Frame Time: 0.033333333333
    ///     0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0
    ///     1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0
    ///     2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0
    /// };
    ///
    /// simple_skeleton
    ///     .frame_cursor()
    ///     .remove_frame()
    ///     .expect("Could not remove frame");
    ///
    /// assert_eq!(simple_skeleton.frames().len(), 2);
    ///
    /// # for (i, frame) in simple_skeleton.frames().enumerate() {
    /// #     for motion in &frame.as_slice()[..] {
    /// #         assert_eq!(*motion, (i + 1) as f32);
    /// #     }
    /// # }
    /// # } // fn main()
    /// ```
    pub fn remove_frame(&mut self) -> Result<&mut Self, FrameRemoveError> {
        if self.len() == 0 {
            return Err(FrameRemoveError::new(self.index));
        }

        let bnd = |idx| idx * self.num_channels();
        let frame_bound = bnd(self.index)..bnd(self.index + 1);

        let mut i = 0;
        self.bvh
            .motion_values
            .retain(|_| (!frame_bound.contains(&i), i += 1).0);

        Ok(self)
    }

    /// Removes every frame in the [`Bvh`].
    //
    /// # Examples
    ///
    /// ```
    /// # fn main() {
    /// # use bvh_anim::bvh;
    /// let mut simple_skeleton = bvh! {
    ///     HIERARCHY
    ///     ROOT Base
    ///     {
    ///         // Hierarchy omitted...
    ///     #     OFFSET 0.0 0.0 0.0
    ///     #     CHANNELS 6 Xposition Yposition Zposition Zrotation Xrotation Yrotation
    ///     #     JOINT Middle1
    ///     #     {
    ///     #         OFFSET 0.0 0.0 15.0
    ///     #         CHANNELS 3 Zrotation Xrotation Yrotation
    ///     #         JOINT Tip1
    ///     #         {
    ///     #             OFFSET 0.0 0.0 30.0
    ///     #             CHANNELS 3 Zrotation Xrotation Yrotation
    ///     #             End Site
    ///     #             {
    ///     #                 OFFSET 0.0 0.0 45.0
    ///     #             }
    ///     #         }
    ///     #     }
    ///     #     JOINT Middle2
    ///     #     {
    ///     #         OFFSET 0.0 15.0 0.0
    ///     #         CHANNELS 3 Zrotation Xrotation Yrotation
    ///     #         JOINT Tip2
    ///     #         {
    ///     #             OFFSET 0.0 30.0 0.0
    ///     #             CHANNELS 3 Zrotation Xrotation Yrotation
    ///     #             End Site
    ///     #             {
    ///     #                 OFFSET 0.0 45.0 0.0
    ///     #             }
    ///     #         }
    ///     #     }
    ///     }
    ///
    ///     MOTION
    ///     Frames: 3
    ///     Frame Time: 0.033333333333
    ///     0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0
    ///     1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0
    ///     2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0
    /// };
    ///
    /// let mut frame_cursor = simple_skeleton.frame_cursor();
    /// assert_eq!(frame_cursor.len(), 3);
    /// frame_cursor.remove_all_frames();
    /// assert_eq!(frame_cursor.len(), 0);
    /// # } // fn main()
    /// ```
    ///
    /// [`Bvh`]: ../struct.Bvh.html
    #[inline]
    pub fn remove_all_frames(&mut self) -> &mut Self {
        self.bvh.motion_values.clear();
        self
    }

    /// Shrinks the capacity of the motion values in the [`Bvh`] as much as possible.
    ///
    /// [`Bvh`]: ../struct.Bvh.html
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.bvh.motion_values.shrink_to_fit();
    }

    /// Create a new `Frames` iterator from the `FramesCursor`, starting at the current
    /// index.
    #[inline]
    pub fn into_frames(self) -> Frames<'bvh> {
        // @TODO: Replace with `Iterator::advance_by` when stable.
        let mut frames = self.bvh.frames();
        for _ in 0..self.index {
            if frames.next().is_none() {
                break;
            }
        }

        frames
    }

    /// Create a new `FramesMut` iterator from the `FramesCursor`, starting at the current
    /// index.
    #[inline]
    pub fn into_frames_mut(self) -> FramesMut<'bvh> {
        // @TODO: Replace with `Iterator::advance_by` when stable.
        let mut frames = self.bvh.frames_mut();
        for _ in 0..self.index {
            if frames.next().is_none() {
                break;
            }
        }

        frames
    }
}

impl<'bvh> From<&'bvh mut Bvh> for FrameCursor<'bvh> {
    #[inline]
    fn from(bvh: &'bvh mut Bvh) -> Self {
        Self { bvh, index: 0 }
    }
}

impl<'bvh> From<FrameCursor<'bvh>> for Frames<'bvh> {
    #[inline]
    fn from(cursor: FrameCursor<'bvh>) -> Self {
        cursor.into_frames()
    }
}

impl<'bvh> From<FrameCursor<'bvh>> for FramesMut<'bvh> {
    #[inline]
    fn from(cursor: FrameCursor<'bvh>) -> Self {
        cursor.into_frames_mut()
    }
}

/// A utility function which inserts the `slice` into the `vec` at the given `index`.
///
/// # Panics
///
/// Panics if `index` is out of bounds.
fn vec_insert_slice<T: Clone + Default>(vec: &mut Vec<T>, index: usize, slice: &[T]) {
    assert!(
        index <= vec.len(),
        "attempted to insert a slice at index {} into a vec of length {}",
        index,
        vec.len(),
    );

    let spare_capacity = vec.capacity().saturating_sub(vec.len());
    if slice.len() > spare_capacity {
        vec.reserve(slice.len() - spare_capacity);
    }

    let old_len = vec.len();
    let new_len = vec.len() + slice.len();
    vec.resize_with(new_len, T::default);

    // Move the existing items to the end
    for (to_move, end_index) in (index..old_len).into_iter().zip(old_len + index..new_len) {
        vec.swap(to_move, end_index);
    }

    // Clone the slice into the new area
    for (i, item) in slice.iter().enumerate() {
        vec[i + index] = item.clone();
    }
}

#[test]
fn test_vec_insert_slice() {
    let mut v;
    let sl = &[100, 200, 300];

    v = vec![1, 2, 3];
    vec_insert_slice(&mut v, 0, sl);
    assert_eq!(v, &[100, 200, 300, 1, 2, 3]);

    v = vec![1, 2, 3];
    vec_insert_slice(&mut v, 1, sl);
    assert_eq!(v, &[1, 100, 200, 300, 2, 3]);

    v = vec![1, 2, 3];
    vec_insert_slice(&mut v, 2, sl);
    assert_eq!(v, &[1, 2, 100, 200, 300, 3]);

    v = vec![1, 2, 3];
    vec_insert_slice(&mut v, 3, sl);
    assert_eq!(v, &[1, 2, 3, 100, 200, 300]);
}
