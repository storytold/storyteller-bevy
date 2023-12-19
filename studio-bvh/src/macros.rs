use crate::{
    joint::{JointData, Offset},
    Bvh, Channel, ChannelType,
};

#[doc(hidden)]
#[macro_export]
macro_rules! match_channels {
    ($builder:ident; ) => {
    };
    ($builder:ident; Xposition $($rest:ident)*) => {
        $builder.push_channel($crate::ChannelType::PositionX);
        $crate::match_channels!($builder; $($rest)*);
    };
    ($builder:ident; Yposition $($rest:ident)*) => {
        $builder.push_channel($crate::ChannelType::PositionY);
        $crate::match_channels!($builder; $($rest)*);
    };
    ($builder:ident; Zposition $($rest:ident)*) => {
        $builder.push_channel($crate::ChannelType::PositionZ);
        $crate::match_channels!($builder; $($rest)*);
    };
    ($builder:ident; Xrotation $($rest:ident)*) => {
        $builder.push_channel($crate::ChannelType::RotationX);
        $crate::match_channels!($builder; $($rest)*);
    };
    ($builder:ident; Yrotation $($rest:ident)*) => {
        $builder.push_channel($crate::ChannelType::RotationY);
        $crate::match_channels!($builder; $($rest)*);
    };
    ($builder:ident; Zrotation $($rest:ident)*) => {
        $builder.push_channel($crate::ChannelType::RotationZ);
        $crate::match_channels!($builder; $($rest)*);
    };
    ($builder:expr; $($other:tt)*) => {
        compile_error!("Unknown tokens");
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! parse_offset {
    ($builder:ident ($ofst_x:literal $ofst_y:literal $ofst_z:literal)) => {
        let offset = [f32::from($ofst_x), f32::from($ofst_y), f32::from($ofst_z)];
        $builder.push_joint_offset(offset.into(), false);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! parse_joints_internal {
    ($builder:ident (
        $(
            JOINT $joint_nm:ident
            {
                $( $children:tt )*
            }
        )*
    )) => {
        $(
            $builder.push_joint(stringify!($joint_nm));

            $builder.current_depth += 1;
            $crate::parse_joints_internal!($builder ( $( $children )* ));
            $builder.current_depth -= 1;
        )*
    };

    ($builder:ident (
        OFFSET $ofst_x:literal $ofst_y:literal $ofst_z:literal
        CHANNELS 0
        $($rest:tt)*
    )) => {
        $crate::parse_offset!($builder ($ofst_x $ofst_y $ofst_z));
        $crate::parse_joints_internal!($builder ( $($rest)* ));
    };

    ($builder:ident (
        OFFSET $ofst_x:literal $ofst_y:literal $ofst_z:literal
        CHANNELS 1 $ch0:ident
        $($rest:tt)*
    )) => {
        $crate::parse_offset!($builder ($ofst_x $ofst_y $ofst_z));
        $crate::match_channels!($builder ; $ch0);
        $crate::parse_joints_internal!($builder ( $($rest)* ));
    };

    ($builder:ident (
        OFFSET $ofst_x:literal $ofst_y:literal $ofst_z:literal
        CHANNELS 2 $ch0:ident $ch1:ident
        $($rest:tt)*
    )) => {
        $crate::parse_offset!($builder ($ofst_x $ofst_y $ofst_z));
        $crate::match_channels!($builder ; $ch0 $ch1);
        $crate::parse_joints_internal!($builder ( $($rest)* ));
    };

    ($builder:ident (
        OFFSET $ofst_x:literal $ofst_y:literal $ofst_z:literal
        CHANNELS 3 $ch0:ident $ch1:ident $ch2:ident
        $($rest:tt)*
    )) => {
        $crate::parse_offset!($builder ($ofst_x $ofst_y $ofst_z));
        $crate::match_channels!($builder ; $ch0 $ch1 $ch2);
        $crate::parse_joints_internal!($builder ( $($rest)* ));
    };

    ($builder:ident (
        OFFSET $ofst_x:literal $ofst_y:literal $ofst_z:literal
        CHANNELS 4 $ch0:ident $ch1:ident $ch2:ident $ch3:ident
        $($rest:tt)*
    )) => {
        bvh_anim$crate::parse_offset!($builder ($ofst_x $ofst_y $ofst_z));
        bvh_anim$crate::match_channels!($builder ; $ch0 $ch1 $ch2 $ch3);
        bvh_anim$crate::parse_joints_internal!($builder ( $($rest)* ));
    };

    ($builder:ident (
        OFFSET $ofst_x:literal $ofst_y:literal $ofst_z:literal
        CHANNELS 5
            $ch0:ident
            $ch1:ident
            $ch2:ident
            $ch3:ident
            $ch4:ident
        $($rest:tt)*
    )) => {
        bvh_anim::parse_offset!($builder ($ofst_x $ofst_y $ofst_z));
        bvh_anim::match_channels!($builder ; $ch0 $ch1 $ch2 $ch3 $ch4);
        bvh_anim::parse_joints_internal!($builder ( $($rest)* ));
    };

    ($builder:ident (
        OFFSET $ofst_x:literal $ofst_y:literal $ofst_z:literal
        CHANNELS 6
            $ch0:ident
            $ch1:ident
            $ch2:ident
            $ch3:ident
            $ch4:ident
            $ch5:ident
        $($rest:tt)*
    )) => {
        $crate::parse_offset!($builder ($ofst_x $ofst_y $ofst_z));
        $crate::match_channels!($builder ;
            $ch0 $ch1 $ch2 $ch3 $ch4 $ch5);
        $crate::parse_joints_internal!($builder ( $($rest)* ));
    };

    ($builder:ident (
        CHANNELS 7
            $ch0:ident
            $ch1:ident
            $ch2:ident
            $ch3:ident
            $ch4:ident
            $ch5:ident
            $ch6:ident
        $($rest:tt)*
    )) => {
        $crate::parse_offset!($builder ($ofst_x $ofst_y $ofst_z));
        $crate::match_channels!($builder ;
            $ch0 $ch1 $ch2 $ch3 $ch4 $ch5 $ch6);
        $crate::parse_joints_internal!($builder ( $($rest)* ));
    };

    ($builder:ident (
        OFFSET $ofst_x:literal $ofst_y:literal $ofst_z:literal
        CHANNELS 8
            $ch0:ident
            $ch1:ident
            $ch2:ident
            $ch3:ident
            $ch4:ident
            $ch5:ident
            $ch6:ident
            $ch7:ident
        $($rest:tt)*
    )) => {
        $crate::parse_offset!($builder ($ofst_x $ofst_y $ofst_z));
        $crate::match_channels!($builder ;
            $ch0 $ch1 $ch2 $ch3 $ch4 $ch5 $ch6 $ch7);
        $crate::parse_joints_internal!($builder ( $($rest)* ));
    };

    ($builder:ident (
        OFFSET $ofst_x:literal $ofst_y:literal $ofst_z:literal
        CHANNELS 9
            $ch0:ident
            $ch1:ident
            $ch2:ident
            $ch3:ident
            $ch4:ident
            $ch5:ident
            $ch6:ident
            $ch7:ident
            $ch8:ident
        $($rest:tt)*
    )) => {
        $crate::parse_offset!($builder ($ofst_x $ofst_y $ofst_z));
        $crate::match_channels!($builder ;
            $ch0 $ch1 $ch2 $ch3 $ch4 $ch5 $ch6 $ch7 $ch8);
        $crate::parse_joints_internal!($builder ( $($rest)* ));
    };

    ($builder:ident (
        OFFSET $ofst_x:literal $ofst_y:literal $ofst_z:literal
        CHANNELS $unsupported:literal $($rest:tt)*
    )) => {
        compile_error!("No more than 9 channels supported in CHANNELS statement");
    };

    ($builder:ident (
        End Site
        {
            OFFSET $end_x:literal $end_y:literal $end_z:literal
        }
    )) => {
        let offset = [
            f32::from($end_x),
            f32::from($end_y),
            f32::from($end_z),
        ];

        $builder.push_joint_offset(offset.into() , true);
    };
}

/// Create a new [`Bvh`][`Bvh`] object using a macro literal. Useful for
/// testing.
///
/// # Notes
///
/// If you have a very complex `Bvh` file with a large number of joints and frames, then
/// this macro will scale badly to it, and compilation time will suffer.
///
/// # Example
///
/// ```
/// # use bvh_anim::bvh;
/// let simple_skeleton = bvh! {
///     HIERARCHY
///     ROOT Base
///     {
///         OFFSET 0.0 0.0 0.0
///         CHANNELS 6 Xposition Yposition Zposition Zrotation Xrotation Yrotation
///         JOINT Middle1
///         {
///             OFFSET 0.0 0.0 15.0
///             CHANNELS 3 Zrotation Xrotation Yrotation
///             JOINT Tip1
///             {
///                 OFFSET 0.0 0.0 30.0
///                 CHANNELS 3 Zrotation Xrotation Yrotation
///                 End Site
///                 {
///                     OFFSET 0.0 0.0 45.0
///                 }
///             }
///         }
///         JOINT Middle2
///         {
///             OFFSET 0.0 15.0 0.0
///             CHANNELS 3 Zrotation Xrotation Yrotation
///             JOINT Tip2
///             {
///                 OFFSET 0.0 30.0 0.0
///                 CHANNELS 3 Zrotation Xrotation Yrotation
///                 End Site
///                 {
///                     OFFSET 0.0 45.0 0.0
///                 }
///             }
///         }
///     }
///
///     MOTION
///     Frames: 3
///     // Time in seconds.
///     Frame Time: 0.033333333333
///     0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0
///     1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0 1.0
///     2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0 2.0
/// };
/// ```
///
/// You can use the `bvh` macro to create empty `Bvh` instances:
///
/// ```
/// # use bvh_anim::bvh;
/// let empty = bvh!{};
///
/// let empty = bvh! {
///     HIERARCHY
///     MOTION
/// };
///
/// let empty = bvh! {
///     HIERARCHY
///     MOTION
///     Frames: 0
/// };
/// ```
///
/// [`bvh`]: struct.Bvh.html
#[macro_export]
macro_rules! bvh {
    () => {
        $crate::Bvh::default()
    };

    (
        HIERARCHY
        MOTION
    ) => {
        $crate::Bvh::default()
    };

    (
        HIERARCHY
        MOTION
        Frames: 0
    ) => {
        $crate::Bvh::default()
    };

    (
        HIERARCHY
        MOTION
        Frames: 0
        Frame Time: $frame_time:literal
    ) => {
        {
            use std::time::Duration;
            let mut bvh = $crate::Bvh::default();
            bvh.set_frame_time(Duration::from_secs_f64(f64::from($frame_time)));
            bvh
        }
    };

    (
        HIERARCHY
        ROOT $root_name:ident
        {
            $( $joints:tt )*
        }
        MOTION
        Frames: 0
        Frame Time: $frame_time:literal
    ) => {
        {
            use std::time::Duration;
            use $crate::parse_joints_internal;

            let mut builder = $crate::BvhLiteralBuilder::default();
            builder.push_root(stringify!($root_name));

            builder.current_depth += 1;
            parse_joints_internal!(builder ($($joints)*));
            builder.current_depth -= 1;

            builder.set_num_channels();
            builder.bvh.set_frame_time(Duration::from_secs_f64(f64::from($frame_time)));

            builder.bvh
        }
    };

    (
        HIERARCHY
        ROOT $root_name:ident
        {
            $( $joints:tt )*
        }
        MOTION
        Frames: $num_frames:literal
        Frame Time: $frame_time:literal
        $(
            $motion:literal
        )+
    ) => {
        {
            use $crate::bvh;

            let mut new_bvh = bvh! {
                HIERARCHY
                ROOT $root_name
                {
                    $( $joints )*
                }
                MOTION
                Frames: 0
                Frame Time: $frame_time
            };

            let num_channels = new_bvh.num_channels();

            new_bvh
                .frame_cursor()
                .try_insert_frames([ $( f32::from($motion) ),+ ].chunks(num_channels))
                .expect("Could not create bvh type from macro literal");

            assert_eq!(new_bvh.frames().len(), $num_frames as usize);

            new_bvh
        }
    };
}

// @TODO: refactor this into a general `Builder` so that we have an
// easy interface to get other animation/skeleton data into a `bvh`
// file.
/// Helper struct to build a `Bvh` from the macro without exposing
/// too many internals.
#[doc(hidden)]
#[derive(Default)]
pub struct BvhLiteralBuilder {
    pub bvh: Bvh,
    pub current_channel_index: usize,
    pub current_depth: usize,
    pub current_index: usize,
}

#[doc(hidden)]
impl BvhLiteralBuilder {
    pub fn push_root(&mut self, name: &str) {
        let mut root = JointData::empty_root();
        root.set_name(name);
        self.bvh.joints.push(root);
        self.current_index += 1;
    }

    pub fn push_joint(&mut self, name: &str) {
        // @TODO: make this shared
        #[inline]
        fn get_parent_index(joints: &[JointData], for_depth: usize) -> usize {
            joints
                .iter()
                .rev()
                .find(|jd| jd.depth() == for_depth.saturating_sub(1))
                .and_then(|jd| jd.private_data().map(|p| p.self_index))
                .unwrap_or(0)
        }

        let idx = self.current_index;
        let dpth = self.current_depth;
        let parent = get_parent_index(&self.bvh.joints[..], dpth);

        let mut joint = JointData::empty_child();
        joint.set_name(name);
        if let Some(ref mut private) = joint.private_data_mut() {
            private.self_index = idx;
            private.depth = dpth;
            private.parent_index = parent;
        };

        self.bvh.joints.push(joint);

        self.current_index += 1;
    }

    pub fn push_channel(&mut self, channel: ChannelType) {
        let channel = Channel::new(channel, self.current_channel_index);
        self.last_joint().map(|joint| match *joint {
            JointData::Root {
                ref mut channels, ..
            } => {
                channels.push(channel);
            }
            JointData::Child {
                ref mut channels, ..
            } => {
                channels.push(channel);
            }
        });
        self.current_channel_index += 1;
    }

    pub fn push_joint_offset(&mut self, offset: Offset, is_end_site: bool) {
        self.last_joint().map(|joint| {
            joint.set_offset(offset, is_end_site);
        });
    }

    #[inline]
    pub fn set_num_channels(&mut self) {
        self.bvh.num_channels = self.current_channel_index;
    }

    #[inline]
    fn last_joint(&mut self) -> Option<&mut JointData> {
        self.bvh.joints.last_mut()
    }
}

#[cfg(test)]
mod tests {
    // Needed for macros
    use std::time::Duration;

    #[test]
    fn macro_create() {
        let bvh = bvh! {
            HIERARCHY
            ROOT Base
            {
                OFFSET 0.0 0.0 0.0
                CHANNELS 6 Xposition Yposition Zposition Zrotation Xrotation Yrotation
                JOINT End
                {
                    OFFSET 0.0 0.0 15.0
                    CHANNELS 3 Zrotation Xrotation Yrotation
                    End Site
                    {
                        OFFSET 0.0 0.0 30.0
                    }
                }
            }

            MOTION
            Frames: 1
            Frame Time: 0.033333333333
            0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0
        };

        {
            use super::{ChannelType, JointData, Offset};

            fn check_joint<O: Into<Option<Offset>>>(
                joint: &JointData,
                expected_name: &str,
                expected_offset: Offset,
                channels: &[ChannelType],
                end_site: O,
            ) {
                assert_eq!(joint.name(), expected_name);
                assert_eq!(*joint.offset(), expected_offset);
                for (chan, expected_chan) in joint
                    .channels()
                    .iter()
                    .map(|c| c.channel_type())
                    .zip(channels.iter())
                {
                    assert_eq!(chan, *expected_chan);
                }
                let end_site = end_site.into().map(Into::into);
                assert_eq!(joint.end_site(), end_site.as_ref());
            }

            let mut joints = bvh.joints();

            check_joint(
                joints.next().unwrap().data(),
                "Base",
                [0.0, 0.0, 0.0],
                &[
                    ChannelType::PositionX,
                    ChannelType::PositionY,
                    ChannelType::PositionZ,
                    ChannelType::RotationZ,
                    ChannelType::RotationX,
                    ChannelType::RotationY,
                ],
                None,
            );

            check_joint(
                joints.next().unwrap().data(),
                "End",
                [0.0, 0.0, 15.0],
                &[
                    ChannelType::RotationZ,
                    ChannelType::RotationX,
                    ChannelType::RotationY,
                ],
                [0.0, 0.0, 30.0],
            );
        }

        assert_eq!(*bvh.frame_time(), Duration::from_nanos(33333333));

        let frames = bvh.frames();
        assert_eq!(frames.len(), 1);
        for frame in frames {
            assert_eq!(frame.len(), 9);
            for channel in frame.as_slice().iter() {
                assert_eq!(*channel, 0.0);
            }
        }
    }

    #[test]
    fn test_empty_create() {
        macro_rules! assert_empty {
            ($bvh:expr) => {
                assert!($bvh.joints().next().is_none());
                assert_eq!(*$bvh.frame_time(), Duration::default());
                assert!($bvh.frames().next().is_none());
            };
        }

        let empty = bvh! {};
        assert_empty!(empty);

        let empty_2 = bvh! {
            HIERARCHY
            MOTION
        };
        assert_empty!(empty_2);
    }
}
