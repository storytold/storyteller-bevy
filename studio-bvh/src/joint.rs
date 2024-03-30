#![allow(dead_code, missing_docs)]

use crate::Channel;
use bstr::{BStr, BString, ByteSlice, Utf8Error};
use smallvec::SmallVec;
use std::{
    cmp::{Ordering, PartialEq, PartialOrd},
    collections::VecDeque,
    ffi::{CStr, CString},
    fmt, mem,
    ops::{Deref, DerefMut},
    str,
};

/// A 3-element point representing the location of a joint offset or
/// end site.
pub type Offset = [f32; 3];

/// Internal representation of a joint.
#[derive(Clone, Debug, PartialEq)]
pub enum JointData {
    /// Root of the skeletal heirarchy.
    Root {
        /// Name of the root `Joint`.
        name: JointName,
        /// Positional offset of this `Joint` relative to the parent.
        offset: Offset,
        /// The channels applicable to this `Joint`.
        channels: SmallVec<[Channel; 6]>,
    },
    /// A child joint in the skeleton.
    Child {
        /// Name of the `Joint`.
        name: JointName,
        /// Positional offset of this `Joint` relative to the parent.
        offset: Offset,
        /// The channels applicable to this `Joint`.
        channels: SmallVec<[Channel; 3]>,
        /// End site offset.
        end_site_offset: Option<Offset>,
        /// Private data.
        private: JointPrivateData,
    },
}

impl JointData {
    #[inline]
    pub const fn is_root(&self) -> bool {
        matches!(*self, JointData::Root { .. })
    }

    #[inline]
    pub const fn is_child(&self) -> bool {
        matches!(*self, JointData::Child { .. })
    }

    #[inline]
    pub fn name(&self) -> &[u8] {
        let name = match *self {
            JointData::Root { ref name, .. } | JointData::Child { ref name, .. } => name,
        };

        name.as_bytes()
    }

    #[inline]
    pub fn name_str(&self) -> Result<&str, Utf8Error> {
        let name = match *self {
            JointData::Root { ref name, .. } => name,
            JointData::Child { ref name, .. } => name,
        };

        name.as_str()
    }

    #[inline]
    pub const fn offset(&self) -> &Offset {
        match *self {
            JointData::Child { ref offset, .. } | JointData::Root { ref offset, .. } => offset,
        }
    }

    #[inline]
    pub const fn end_site(&self) -> Option<&Offset> {
        match *self {
            JointData::Child {
                ref end_site_offset,
                ..
            } => end_site_offset.as_ref(),
            _ => None,
        }
    }

    #[inline]
    pub fn channels(&self) -> &[Channel] {
        match *self {
            JointData::Child { ref channels, .. } => &channels[..],
            JointData::Root { ref channels, .. } => &channels[..],
        }
    }

    /// Returns a mutable reference to ordered array of `Channel`s of this `JointData`.
    #[inline]
    pub fn channels_mut(&mut self) -> &mut [Channel] {
        match *self {
            JointData::Child {
                ref mut channels, ..
            } => &mut channels[..],
            JointData::Root {
                ref mut channels, ..
            } => &mut channels[..],
        }
    }

    /// Return the index of this `Joint` in the array.
    #[inline]
    pub fn index(&self) -> usize {
        self.private_data().map(|d| d.self_index).unwrap_or(0)
    }

    /// Returns the index of this joint's parent in the `Bvh`'s array of joints,
    /// or `None` if this `Joint` is the root joint.
    #[inline]
    pub fn parent_index(&self) -> Option<usize> {
        self.private_data().map(|d| d.parent_index)
    }

    /// Returns a reference to the `JointPrivateData` of the `JointData` if it
    /// exists, or `None`.
    #[inline]
    pub(crate) const fn private_data(&self) -> Option<&JointPrivateData> {
        match *self {
            JointData::Child { ref private, .. } => Some(private),
            _ => None,
        }
    }

    /// Get the depth of the `JointData` in the heirarchy.
    #[inline]
    pub(crate) const fn depth(&self) -> usize {
        match *self {
            JointData::Child { ref private, .. } => private.depth,
            _ => 0,
        }
    }

    #[inline]
    pub(crate) fn empty_root() -> Self {
        JointData::Root {
            name: Default::default(),
            offset: [0.0, 0.0, 0.0],
            channels: Default::default(),
        }
    }

    #[inline]
    pub(crate) fn empty_child() -> Self {
        JointData::Child {
            name: Default::default(),
            offset: [0.0, 0.0, 0.0],
            channels: Default::default(),
            end_site_offset: Default::default(),
            private: JointPrivateData::empty(),
        }
    }

    #[inline]
    pub(crate) fn set_name<S: Into<JointName>>(&mut self, new_name: S) {
        match *self {
            JointData::Root { ref mut name, .. } => *name = new_name.into(),
            JointData::Child { ref mut name, .. } => *name = new_name.into(),
        }
    }

    pub(crate) fn set_offset(&mut self, new_offset: impl Into<Offset>, is_site: bool) {
        let new_offset = new_offset.into();
        match *self {
            JointData::Root { ref mut offset, .. } => *offset = new_offset,
            JointData::Child {
                ref mut offset,
                ref mut end_site_offset,
                ..
            } => {
                if is_site {
                    *end_site_offset = Some(new_offset);
                } else {
                    *offset = new_offset;
                }
            }
        }
    }

    #[inline]
    pub(crate) fn set_channels(&mut self, new_channels: SmallVec<[Channel; 6]>) {
        match *self {
            JointData::Root {
                ref mut channels, ..
            } => *channels = new_channels,
            JointData::Child {
                ref mut channels, ..
            } => *channels = new_channels.iter().copied().collect(),
        }
    }

    /// Returns a mutable reference to the `JointPrivateData` of the `JointData` if it
    /// exists, or `None`.
    #[inline]
    pub(crate) fn private_data_mut(&mut self) -> Option<&mut JointPrivateData> {
        match *self {
            JointData::Child {
                ref mut private, ..
            } => Some(private),
            _ => None,
        }
    }
}

/// A string type for the `Joint` name. A `SmallVec` is used for
/// better data locality.
pub(crate) type JointNameInner = SmallVec<[u8; mem::size_of::<String>()]>;

/// Wrapper struct for the `Joint` name type.
#[derive(Clone, Default, Eq, Hash, Ord)]
pub struct JointName(pub(crate) JointNameInner);

impl JointName {
    #[inline]
    pub(crate) fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }

    #[inline]
    pub(crate) fn as_str(&self) -> Result<&str, Utf8Error> {
        self.as_bytes().to_str()
    }
}

impl Deref for JointName {
    type Target = JointNameInner;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for JointName {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<B: ?Sized + AsRef<[u8]>> PartialEq<B> for JointName {
    #[inline]
    fn eq(&self, rhs: &B) -> bool {
        AsRef::<[u8]>::as_ref(self) == rhs.as_ref()
    }
}

impl<B: ?Sized + AsRef<[u8]>> PartialOrd<B> for JointName {
    #[inline]
    fn partial_cmp(&self, rhs: &B) -> Option<Ordering> {
        AsRef::<[u8]>::as_ref(self).partial_cmp(rhs.as_ref())
    }
}

macro_rules! impl_from {
    () => {};
    (;) => {};
    ($t:ty; $($rest:tt)*) => {
        impl From<$t> for JointName {
            #[inline]
            fn from(b: $t) -> Self {
                JointName(b.bytes().collect())
            }
        }
        impl_from!($($rest)*);
    }
}

impl From<CString> for JointName {
    #[inline]
    fn from(s: CString) -> Self {
        From::from(s.into_bytes())
    }
}

impl From<&'_ CStr> for JointName {
    #[inline]
    fn from(s: &'_ CStr) -> Self {
        From::from(s.to_bytes())
    }
}

impl From<Vec<u8>> for JointName {
    #[inline]
    fn from(s: Vec<u8>) -> Self {
        JointName(JointNameInner::from(s))
    }
}

impl From<&'_ [u8]> for JointName {
    #[inline]
    fn from(s: &'_ [u8]) -> Self {
        JointName(s.iter().copied().collect::<JointNameInner>())
    }
}

impl_from! {
    String; &'_ str;
    BString; &'_ BStr;
}

macro_rules! impl_as_ref {
    ($( $t:ty { ref => $method:path, mut => $mut_method:path } )*) => {
        $(
            impl AsRef<$t> for JointName {
                #[inline]
                fn as_ref(&self) -> &$t {
                    $method(&self.0[..])
                }
            }
            impl AsMut<$t> for JointName {
                #[inline]
                fn as_mut(&mut self) -> &mut $t {
                    $mut_method(&mut self.0[..])
                }
            }
        )*
    }
}

impl_as_ref! {
    BStr { ref => AsRef::<BStr>::as_ref, mut => AsMut::<BStr>::as_mut }
    [u8] { ref => AsRef::<[u8]>::as_ref, mut => AsMut::<[u8]>::as_mut }
}

impl From<JointNameInner> for JointName {
    #[inline]
    fn from(s: JointNameInner) -> Self {
        JointName(s)
    }
}

impl From<JointName> for JointNameInner {
    #[inline]
    fn from(j: JointName) -> Self {
        j.0
    }
}

impl fmt::Debug for JointName {
    #[inline]
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(AsRef::<BStr>::as_ref(self), fmtr)
    }
}

impl fmt::Display for JointName {
    #[inline]
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(AsRef::<BStr>::as_ref(self), fmtr)
    }
}

/// Data private to joints.
#[doc(hidden)]
#[derive(Clone, Eq, PartialEq)]
pub struct JointPrivateData {
    /// Index of this `Joint` in the array.
    pub(crate) self_index: usize,
    /// The parent index in the array of `JointPrivateData`s in the `Bvh`.
    pub(crate) parent_index: usize,
    /// Depth of the `Joint`. A depth of `1` signifies a `Joint` attached to
    /// the root.
    pub(crate) depth: usize,
}

impl JointPrivateData {
    #[inline]
    pub(crate) const fn new(self_index: usize, parent_index: usize, depth: usize) -> Self {
        JointPrivateData {
            self_index,
            parent_index,
            depth,
        }
    }

    #[inline]
    pub(crate) const fn empty() -> Self {
        Self::new(0, 0, 0)
    }
}

impl fmt::Debug for JointPrivateData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("JointPrivateData { .. }")
    }
}

pub(crate) enum JointIterType {
    All(usize),
    Sequence(VecDeque<usize>),
}

/// An iterator over the `Joint`s of a `Bvh` skeleton.
pub struct Joints<'a> {
    pub(crate) joints: &'a [JointData],
    // pub(crate) motion_values: &'a [f32],
    pub(crate) iter_type: JointIterType,
}

impl fmt::Debug for Joints<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Joints { .. }")
    }
}

impl<'a> Joints<'a> {
    /// Create a `Joints` iterator over all the `joints` in a `Bvh` file.
    pub(crate) const fn iter_root(joints: &'a [JointData]) -> Self {
        Joints {
            joints,
            // clips,
            iter_type: JointIterType::All(0),
        }
    }

    pub(crate) const fn iter_from(joints: &'a [JointData], start: usize) -> Self {
        Joints {
            joints,
            // clips,
            iter_type: JointIterType::All(start),
        }
    }

    /// Create a `Joints` iterator over all the child joints of `joint`.
    ///
    /// # Notes
    ///
    /// This function only iterates over the direct children of `joint`. If you
    /// need to iterate through to the end sites of all children, you will
    /// need to continually call `iter_children` on each `Joint` in the iterator.
    pub(crate) fn iter_children(joint: &Joint<'a>) -> Self {
        let first_child: VecDeque<usize> = joint
            .joints
            .iter()
            .filter(|jd| {
                if let Some(p) = jd.private_data() {
                    p.parent_index == joint.index
                } else {
                    false
                }
            })
            .map(|value| value.index())
            .collect();

        Joints {
            joints: joint.joints,
            iter_type: JointIterType::Sequence(first_child),
        }
    }

    /// Finds the `Joint` named `joint_name`, or `None` if it doesn't exist.
    #[inline]
    pub fn find_by_name<B>(&mut self, joint_name: &B) -> Option<Joint<'a>>
    where
        B: ?Sized + AsRef<[u8]>,
    {
        self.find(|b| b.data().name() == joint_name.as_ref())
    }

    #[allow(unused)]
    pub(crate) fn nth_child(joint: &Joint<'a>, child: usize) -> Option<usize> {
        Joints::iter_children(joint)
            .nth(child)
            .map(|joint| joint.index)
    }
}

impl<'a> Iterator for Joints<'a> {
    type Item = Joint<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.iter_type {
            JointIterType::All(current) => {
                if *current >= self.joints.len() {
                    return None;
                }

                Some(Joint {
                    index: *current,
                    joints: self.joints,
                })
            }
            JointIterType::Sequence(ref mut joints) => {
                let next = joints.pop_front()?;

                if next >= self.joints.len() {
                    return None;
                }

                Some(Joint {
                    index: next,
                    joints: self.joints,
                })
            }
        }
    }
}

/// A mutable iterator over the `Joint`s of a `Bvh` skeleton.
#[allow(unused)]
pub struct JointsMut<'a> {
    pub(crate) joints: &'a mut [JointData],
    pub(crate) current_joint: usize,
    pub(crate) from_child: Option<usize>,
}

impl<'a> JointsMut<'a> {
    pub(crate) fn iter_root(joints: &'a mut [JointData]) -> Self {
        JointsMut {
            joints,
            // clips,
            current_joint: 0,
            from_child: None,
        }
    }
}

impl<'a> Iterator for JointsMut<'a> {
    type Item = JointMut<'a>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl fmt::Debug for JointsMut<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("JointsMut { .. }")
    }
}

/// A view of a joint which provides access to various relevant data.
pub struct Joint<'a> {
    /// Index of the `Joint` in the skeleton.
    pub(crate) index: usize,
    /// `Joints` array which the joint is part of.
    pub(crate) joints: &'a [JointData],
}

impl PartialEq for Joint<'_> {
    #[inline]
    fn eq(&self, rhs: &Self) -> bool {
        let (d0, d1) = (self.data(), rhs.data());
        PartialEq::eq(d0, d1)
    }
}

impl Joint<'_> {
    /// Returns `true` if the `Joint` is the root `Joint`, or `false` if it isn't.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use studio_bvh::bvh;
    /// let bvh = bvh! {
    ///     HIERARCHY
    ///     ROOT Hips
    ///     {
    ///         OFFSET 0.0 0.0 0.0
    ///         CHANNELS 6 Xposition Yposition Zposition Zrotation Xrotation Yrotation
    ///         End Site {
    ///             OFFSET 0.0 0.0 30.0
    ///         }
    ///     }
    ///     MOTION
    ///     Frames: 0
    ///     Frame Time: 0.0333333
    /// };
    ///
    /// let root = bvh.root_joint().unwrap();
    /// assert!(root.is_root());
    /// ```
    #[inline]
    pub const fn is_root(&self) -> bool {
        self.data().is_root()
    }

    /// Returns `true` if the `Joint` is a child `Joint`, or `false` if it isn't.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use studio_bvh::bvh;
    /// let bvh = bvh! {
    /// #   HIERARCHY
    /// #   ROOT Base
    /// #   {
    /// #       OFFSET 0.0 0.0 0.0
    /// #       CHANNELS 6 Xposition Yposition Zposition Zrotation Xrotation Yrotation
    /// #       JOINT End
    /// #       {
    /// #           OFFSET 0.0 0.0 15.0
    /// #           CHANNELS 3 Zrotation Xrotation Yrotation
    /// #           End Site
    /// #           {
    /// #               OFFSET 0.0 0.0 30.0
    /// #           }
    /// #       }
    /// #   }
    /// #   MOTION
    /// #   Frames: 0
    /// #   Frame Time: 0.0333333
    ///     // bvh hierarchy unspecified ..
    /// };
    ///
    /// for joint in bvh.joints().skip(1) {
    ///     assert!(joint.is_child());
    /// }
    /// ```
    #[inline]
    pub const fn is_child(&self) -> bool {
        self.data().is_child()
    }

    /// Returns the name of the `JointData`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use studio_bvh::bvh;
    /// let bvh = bvh! {
    ///     HIERARCHY
    ///     ROOT Hips
    ///     {
    ///         OFFSET 0.0 0.0 0.0
    ///         CHANNELS 6 Xposition Yposition Zposition Zrotation Xrotation Yrotation
    ///         // ...
    /// #       End Site {
    /// #           OFFSET 0.0 0.0 30.0
    /// #       }
    ///     }
    ///     MOTION
    ///     // ...
    /// #   Frames: 0
    /// #   Frame Time: 0.0333333
    /// };
    ///
    /// let root = bvh.root_joint().unwrap();
    /// assert_eq!(root.name(), b"Hips");
    /// ```
    #[inline]
    pub fn name(&self) -> &[u8] {
        self.data().name()
    }

    /// Returns the offset of the `JointData` if it exists, or `None`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use studio_bvh::bvh;
    /// let bvh = bvh! {
    ///     HIERARCHY
    ///     ROOT Hips
    ///     {
    ///         OFFSET 1.2 3.4 5.6
    ///         CHANNELS 6 Xposition Yposition Zposition Zrotation Xrotation Yrotation
    ///         // ...
    /// #       End Site {
    /// #           OFFSET 0.0 0.0 30.0
    /// #       }
    ///     }
    ///     MOTION
    ///     // ...
    /// #   Frames: 0
    /// #   Frame Time: 0.0333333
    /// };
    ///
    /// let root = bvh.root_joint().unwrap();
    /// assert_eq!(root.offset(), &[1.2, 3.4, 5.6]);
    /// ```
    #[inline]
    pub const fn offset(&self) -> &Offset {
        self.data().offset()
    }

    /// Returns the `end_site` position if this `Joint` has an end site, or `None` if
    /// it doesn't.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use studio_bvh::bvh;
    /// let bvh = bvh! {
    ///     HIERARCHY
    ///     ROOT Base
    ///     {
    ///         OFFSET 0.0 0.0 0.0
    ///         CHANNELS 6 Xposition Yposition Zposition Zrotation Xrotation Yrotation
    ///         JOINT Tip
    ///         {
    ///             OFFSET 0.0 0.0 5.0
    ///             CHANNELS 3 Zrotation Xrotation Yrotation
    ///             End Site
    ///             {
    ///                 OFFSET 5.4 3.2 1.0
    ///             }
    ///         }
    ///     }
    ///     MOTION
    ///     // ...
    /// #   Frames: 0
    /// #   Frame Time: 0.0333333
    /// };
    /// let mut joints = bvh.joints();
    /// let base = joints.next().unwrap();
    /// assert!(base.end_site().is_none());
    ///
    /// let tip = joints.next().unwrap();
    /// assert_eq!(tip.end_site(), Some([5.4, 3.2, 1.0].into()).as_ref());
    /// ```
    #[inline]
    pub const fn end_site(&self) -> Option<&Offset> {
        self.data().end_site()
    }

    /// Returns `true` if the `Joint` has an `end_site_offset`, or `false` if it doesn't.
    #[inline]
    pub const fn has_end_site(&self) -> bool {
        self.end_site().is_some()
    }

    /// Returns the ordered array of `Channel`s of this `JointData`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use studio_bvh::{bvh, ChannelType};
    /// let bvh = bvh! {
    ///     HIERARCHY
    ///     ROOT Hips
    ///     {
    ///         OFFSET 0.0 0.0 0.0
    ///         CHANNELS 6 Xposition Yposition Zposition Zrotation Xrotation Yrotation
    ///         // ...
    /// #       End Site {
    /// #           OFFSET 0.0 0.0 30.0
    /// #       }
    ///     }
    ///     MOTION
    ///     // ...
    /// #   Frames: 0
    /// #   Frame Time: 0.0333333
    /// };
    ///
    /// let root = bvh.root_joint().unwrap();
    /// let expected_channels = &[
    ///     ChannelType::PositionX,
    ///     ChannelType::PositionY,
    ///     ChannelType::PositionZ,
    ///     ChannelType::RotationZ,
    ///     ChannelType::RotationX,
    ///     ChannelType::RotationY,
    /// ];
    ///
    /// for (channel, &expected) in root
    ///     .channels()
    ///     .iter()
    ///     .map(|c| c.channel_type())
    ///     .zip(expected_channels.iter())
    /// {
    ///     assert_eq!(channel, expected);
    /// }
    /// ```
    #[inline]
    pub fn channels(&self) -> &[Channel] {
        self.data().channels()
    }

    /// Returns the index of this joint in the `Bvh`'s array of joints.
    #[inline]
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Returns the index of this joint's parent in the `Bvh`'s array of joints,
    /// or `None` if this `Joint` is the root joint.
    #[inline]
    pub fn parent_index(&self) -> Option<usize> {
        self.data().parent_index()
    }

    /// Return the parent `Joint` if it exists, or `None` if it doesn't.
    #[inline]
    pub fn parent(&self) -> Option<Joint<'_>> {
        self.data().parent_index().map(|idx| Joint {
            index: idx,
            joints: self.joints,
        })
    }

    /// Returns an iterator over the children of `self`.
    #[inline]
    pub fn children(&self) -> Joints<'_> {
        Joints::iter_children(self)
    }

    /// Access a read-only view of the internal data of the `Joint`.
    #[inline]
    pub const fn data(&self) -> &JointData {
        &self.joints[self.index]
    }
}

impl fmt::Debug for Joint<'_> {
    #[inline]
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmtr.debug_struct("Joint")
            .field("index", &self.index)
            .field("data", &self.data())
            .finish()
    }
}

/// A view of a joint which provides mutable access.
pub struct JointMut<'a> {
    /// Index of the `Joint` in the skeleton.
    pub(crate) index: usize,
    /// `Joints` array which the joint is part of.
    pub(crate) joints: &'a mut [JointData],
}

impl<'a> JointMut<'a> {
    /*
    /// Return the parent `Joint` if it exists, or `None` if it doesn't.
    #[inline]
    pub fn parent(&self) -> Option<Joint<'_>> {
        self.data().parent_index().map(|idx| Joint {
            self_index: idx,
            skeleton: self.skeleton,
            clips: self.clips,
        })
    }

    pub fn

    /// Returns an iterator over the children of `self`.
    #[inline]
    pub fn children(&self) -> Joints<'_> {
        Joints::iter_children(&self)
    }

    /// Access a read-only view of the internal data of the `Joint`.
    #[inline]
    pub fn data(&self) -> &JointData {
        &self.joints[self_index]
    }
    */
    /// Mutable access to the internal data of the `JointMut`.
    #[inline]
    pub(crate) fn data_mut(&mut self) -> &mut JointData {
        &mut self.joints[self.index]
    }
}
