//! Contains options for `bvh` file formatting.

use crate::{frames::Frames, Bvh, Joint, Joints};
use smallvec::SmallVec;
use std::{
    fmt,
    io::{self, Write},
    iter, mem,
    num::NonZeroUsize,
};

/// Specify formatting options for writing a `Bvh`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct WriteOptions {
    /// Which indentation style to use for nested bones.
    pub indent: IndentStyle,
    /// Which style new line terminator to use when writing the `bvh`.
    pub line_terminator: LineTerminator,
    /// Number of significant figures to use when writing `OFFSET` values.
    ///
    /// If this is `None`, then the minimum precision required will be used
    pub offset_significant_figures: Option<usize>,
    /// Number of significant figures to use when writing the `Frame Time` value.
    ///
    /// If this is `None`, then the minimum precision required will be used
    pub frame_time_significant_figures: Option<usize>,
    /// Number of significant figures to use when writing `MOTION` values.
    ///
    /// If this is `None`, then the minimum precision required will be used
    pub motion_values_significant_figures: Option<usize>,
    #[doc(hidden)]
    _nonexhaustive: (),
}

impl Default for WriteOptions {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl WriteOptions {
    /// Create a new `WriteOptions` with default values.
    #[inline]
    pub const fn new() -> Self {
        WriteOptions {
            indent: IndentStyle::Tabs,
            line_terminator: LineTerminator::native(),
            offset_significant_figures: None,
            frame_time_significant_figures: None,
            motion_values_significant_figures: None,
            _nonexhaustive: (),
        }
    }

    /// Output the `Bvh` file to the `writer` with the given options.
    pub fn write<W: Write>(&self, bvh: &Bvh, writer: &mut W) -> io::Result<()> {
        let mut curr_chunk = vec![];
        let mut curr_bytes_written = 0usize;
        let mut curr_string_len = 0usize;
        let mut iter_state = WriteOptionsIterState::new();

        while self.next_chunk(bvh, &mut curr_chunk, &mut iter_state) != false {
            let bytes: &[u8] = curr_chunk.as_ref();
            curr_string_len += bytes.len();
            curr_bytes_written += writer.write(bytes)?;

            if curr_bytes_written != curr_string_len {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Data has been dropped while writing to file",
                ));
            }
        }
        writer.flush()
    }

    /// Output the `Bvh` file to the `string` with the given options.
    pub fn write_to_string(&self, bvh: &Bvh) -> Vec<u8> {
        let mut curr_chunk = vec![];
        let mut out_string = vec![];
        let mut iter_state = WriteOptionsIterState::new();

        while self.next_chunk(bvh, &mut curr_chunk, &mut iter_state) != false {
            out_string.extend(curr_chunk.drain(..));
        }

        out_string
    }

    /// Sets `indent` on `self` to the new `IndentStyle`.
    #[inline]
    pub const fn with_indent(self, indent: IndentStyle) -> Self {
        Self { indent, ..self }
    }

    /// Sets `line_terminator` on `self` to the new `LineTerminator`.
    #[inline]
    pub const fn with_line_terminator(self, line_terminator: LineTerminator) -> Self {
        Self {
            line_terminator,
            ..self
        }
    }

    /// Sets `offset_significant_figures` on `self` to the new `offset_significant_figures`.
    #[inline]
    pub fn with_offset_significant_figures<N>(self, offset_significant_figures: N) -> Self
    where
        N: Into<Option<usize>>,
    {
        Self {
            offset_significant_figures: offset_significant_figures.into(),
            ..self
        }
    }

    /// Sets `motion_values_significant_figures` on `self` to the new `motion_values_significant_figures`.
    #[inline]
    pub fn with_frame_time_significant_figures<N>(self, frame_time_significant_figures: N) -> Self
    where
        N: Into<Option<usize>>,
    {
        Self {
            frame_time_significant_figures: frame_time_significant_figures.into(),
            ..self
        }
    }

    /// Sets `motion_values_significant_figures` on `self` to the new `motion_values_significant_figures`.
    #[inline]
    pub fn with_motion_values_significant_figures<N>(
        self,
        motion_values_significant_figures: N,
    ) -> Self
    where
        N: Into<Option<usize>>,
    {
        Self {
            motion_values_significant_figures: motion_values_significant_figures.into(),
            ..self
        }
    }

    // @TODO: Refactor all of this
    /// Get the next text chunk of the written bvh file. This function is
    /// structured so that the `chunk` string can be continually
    /// re-used without allocating and de-allocating memory.
    ///
    /// # Returns
    ///
    /// Returns `true` when there are still more lines available,
    /// `false` when all lines have been extracted.
    fn next_chunk<'a, 'b: 'a>(
        &self,
        bvh: &'b Bvh,
        chunk: &mut Vec<u8>,
        iter_state: &'a mut WriteOptionsIterState<'b>,
    ) -> bool {
        chunk.clear();

        let terminator = self.line_terminator.as_bytes();

        match *iter_state {
            WriteOptionsIterState::WriteHierarchy { ref mut written } => {
                if !*written {
                    *chunk = b"HIERARCHY".to_vec();
                    chunk.extend_from_slice(self.line_terminator.as_bytes());
                    *written = true;
                } else {
                    let mut joints = bvh.joints();
                    *iter_state = WriteOptionsIterState::WriteJoints {
                        current_joint: joints.next(),
                        joints,
                        wrote_name: false,
                        wrote_offset: false,
                        wrote_channels: false,
                    };
                }
            }
            WriteOptionsIterState::WriteJoints {
                current_joint: None,
                ..
            } => {
                *iter_state = WriteOptionsIterState::WriteMotion { written: false };
            }
            WriteOptionsIterState::WriteJoints {
                ref mut joints,
                ref mut current_joint,
                ref mut wrote_name,
                ref mut wrote_offset,
                ref mut wrote_channels,
            } => {
                if let Some(ref joint) = current_joint {
                    let joint_data = joint.data();
                    let mut depth = joint_data.depth();
                    if *wrote_name {
                        depth += 1
                    }

                    match (&mut *wrote_name, &mut *wrote_offset, &mut *wrote_channels) {
                        (&mut false, _, _) => {
                            chunk.extend(self.indent.prefix_chars(depth));
                            if joint_data.is_root() {
                                chunk.extend_from_slice(b"ROOT ");
                            } else {
                                chunk.extend_from_slice(b"JOINT ");
                            }
                            chunk.extend_from_slice(joint_data.name().as_ref());
                            chunk.extend_from_slice(terminator);
                            chunk.extend(self.indent.prefix_chars(depth));
                            chunk.push(b'{');
                            chunk.extend_from_slice(terminator);

                            *wrote_name = true;
                        }
                        (&mut true, &mut false, _) => {
                            chunk.extend(self.indent.prefix_chars(depth));

                            let [x, y, z] = joint_data.offset();
                            let offset_str = match self.offset_significant_figures {
                                Some(sf) => {
                                    format!("OFFSET {:.*} {:.*} {:.*}", sf, x, sf, y, sf, z,)
                                }
                                None => format!("OFFSET {:.} {:.} {:.}", x, y, z),
                            };
                            chunk.extend_from_slice(offset_str.as_bytes());
                            chunk.extend_from_slice(terminator);
                            *wrote_offset = true;
                        }
                        (&mut true, &mut true, &mut false) => {
                            chunk.extend(self.indent.prefix_chars(depth));

                            let channels = joint_data.channels();
                            let channels_str = channels
                                .iter()
                                .map(|ch| ch.channel_type().as_str())
                                .collect::<SmallVec<[_; 6]>>()
                                .join(" ");

                            let channels_str =
                                format!("CHANNELS {} {}", channels.len(), channels_str);
                            chunk.extend_from_slice(channels_str.as_bytes());
                            chunk.extend_from_slice(terminator);
                            *wrote_channels = true;
                        }
                        (&mut true, &mut true, &mut true) => {
                            if let Some([x, y, z]) = joint_data.end_site() {
                                chunk.extend(self.indent.prefix_chars(depth));
                                chunk.extend_from_slice(b"End Site");
                                chunk.extend_from_slice(terminator);

                                chunk.extend(self.indent.prefix_chars(depth));
                                chunk.push(b'{');
                                chunk.extend_from_slice(terminator);

                                chunk.extend(self.indent.prefix_chars(depth + 1));
                                let offset_str = match self.offset_significant_figures {
                                    Some(sf) => {
                                        format!("OFFSET {:.*} {:.*} {:.*}", sf, x, sf, y, sf, z)
                                    }
                                    None => format!("OFFSET {:.} {:.} {:.}", x, y, z),
                                };
                                chunk.extend_from_slice(offset_str.as_bytes());
                                chunk.extend_from_slice(terminator);

                                chunk.extend(self.indent.prefix_chars(depth));
                                chunk.push(b'}');
                                chunk.extend_from_slice(terminator);

                                let next_joint = joints.next();
                                let prev_joint = mem::replace(current_joint, next_joint).unwrap();

                                let (curr_depth, mut depth_difference) =
                                    if let Some(ref curr_j) = *current_joint {
                                        let curr_depth = curr_j.data().depth();
                                        (curr_depth, Some(prev_joint.data().depth() - curr_depth))
                                    } else {
                                        (0, Some(prev_joint.data().depth()))
                                    };

                                while let Some(d) = depth_difference {
                                    chunk.extend(self.indent.prefix_chars(curr_depth + d));
                                    chunk.push(b'}');
                                    chunk.extend_from_slice(terminator);
                                    depth_difference =
                                        depth_difference.and_then(|d| d.checked_sub(1));
                                }
                            } else {
                                *current_joint = joints.next();
                            }
                            *wrote_name = false;
                            *wrote_offset = false;
                            *wrote_channels = false;
                        }
                    }
                }
            }
            WriteOptionsIterState::WriteMotion { ref mut written } => {
                if !*written {
                    *chunk = b"MOTION".to_vec();
                    chunk.extend_from_slice(terminator);
                    *written = true;
                } else {
                    *iter_state = WriteOptionsIterState::WriteNumFrames { written: false };
                }
            }
            WriteOptionsIterState::WriteNumFrames { ref mut written } => {
                if !*written {
                    *chunk = format!("Frames: {}", bvh.frames().len()).into_bytes();
                    chunk.extend_from_slice(terminator);
                    *written = true;
                } else {
                    *iter_state = WriteOptionsIterState::WriteFrameTime { written: false };
                }
            }
            WriteOptionsIterState::WriteFrameTime { ref mut written } => {
                if !*written {
                    *chunk = match self.frame_time_significant_figures {
                        Some(sf) => {
                            format!("Frame Time: {:.*}", sf, bvh.frame_time().as_secs_f64())
                                .into_bytes()
                        }
                        None => {
                            format!("Frame Time: {:.}", bvh.frame_time().as_secs_f64()).into_bytes()
                        }
                    };
                    chunk.extend_from_slice(terminator);
                    *written = true;
                } else {
                    let frames = bvh.frames();
                    *iter_state = WriteOptionsIterState::WriteFrames { frames };
                }
            }
            WriteOptionsIterState::WriteFrames { ref mut frames } => {
                return frames
                    .next()
                    .map(|frame| {
                        let motion_values = frame
                            .as_slice()
                            .iter()
                            .map(|motion| match self.motion_values_significant_figures {
                                Some(sf) => format!("{:.*}", sf, motion),
                                None => format!("{:.}", motion),
                            })
                            .collect::<Vec<_>>()
                            .join(" ");
                        *chunk = motion_values.into_bytes();
                        chunk.extend_from_slice(terminator);
                        true
                    })
                    .unwrap_or_default();
            }
        }

        true
    }
}

enum WriteOptionsIterState<'a> {
    WriteHierarchy {
        written: bool,
    },
    WriteJoints {
        joints: Joints<'a>,
        current_joint: Option<Joint<'a>>,
        wrote_name: bool,
        wrote_offset: bool,
        wrote_channels: bool,
    },
    WriteMotion {
        written: bool,
    },
    WriteNumFrames {
        written: bool,
    },
    WriteFrameTime {
        written: bool,
    },
    WriteFrames {
        frames: Frames<'a>,
    },
}

impl WriteOptionsIterState<'_> {
    #[inline]
    const fn new() -> Self {
        WriteOptionsIterState::WriteHierarchy { written: false }
    }
}

impl Default for WriteOptionsIterState<'_> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Specify indentation style to use when writing the `Bvh` joints.
///
/// By default, this value is set to 1 tab.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum IndentStyle {
    /// Do not indent nested joints.
    NoIndentation,
    /// Use a single tab (`'\t'`) for indentation.
    Tabs,
    /// Use `n` spaces for indentation.
    Spaces(NonZeroUsize),
}

impl IndentStyle {
    /// Create a new `IndentStyle` with `n` preceeding spaces.
    ///
    /// If `n` is `0`, then `IndentStyle::NoIndentation` is returned.
    #[inline]
    pub fn with_spaces(n: usize) -> Self {
        NonZeroUsize::new(n)
            .map(IndentStyle::Spaces)
            .unwrap_or(IndentStyle::NoIndentation)
    }

    /// Return an `Iterator` which yields bytes corresponding to the ascii
    /// chars which form the `String` this indentation style would take.
    #[inline]
    fn prefix_chars(&self, depth: usize) -> impl Iterator<Item = u8> {
        match *self {
            IndentStyle::NoIndentation => iter::repeat(b'\0').take(0),
            IndentStyle::Tabs => iter::repeat(b'\t').take(depth),
            IndentStyle::Spaces(n) => iter::repeat(b' ').take(n.get() * depth),
        }
    }
}

/// Create a new `IndentStyle` using a single tab.
impl Default for IndentStyle {
    #[inline]
    fn default() -> Self {
        IndentStyle::Tabs
    }
}

/// Represents which line terminator style to use when writing a `Bvh` file.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LineTerminator {
    /// Use Unix-style line endings (`'\n'`).
    Unix,
    /// Use Windows-style line endings (`'\r\n'`).
    Windows,
}

impl LineTerminator {
    /// Get the line terminator style native to the current OS:
    ///
    /// * On Windows, this returns `LineTerminator::Windows`.
    /// * Otherwise, this returns `LineTerminator::Unix`.
    #[inline]
    pub const fn native() -> Self {
        if cfg!(windows) {
            LineTerminator::Windows
        } else {
            LineTerminator::Unix
        }
    }

    /// Return the characters of the `LineTerminator` as a `&str`.
    #[inline]
    pub const fn as_str(&self) -> &str {
        match *self {
            LineTerminator::Unix => "\n",
            LineTerminator::Windows => "\r\n",
        }
    }

    /// Returns the escaped characters of the `LineTerminator` as a `&str`.
    #[inline]
    pub const fn as_escaped_str(&self) -> &str {
        match *self {
            LineTerminator::Unix => r"\n",
            LineTerminator::Windows => r"\r\n",
        }
    }

    /// Return the characters of the `LineTerminator` as a `&[u8]`.
    #[inline]
    pub const fn as_bytes(&self) -> &[u8] {
        self.as_str().as_bytes()
    }

    /// Returns the escaped characters of the `LineTerminator` as a `&[u8]`.
    #[inline]
    pub const fn as_escaped_bytes(&self) -> &[u8] {
        self.as_escaped_str().as_bytes()
    }
}

/// Returns the native line terminator for the current OS.
impl Default for LineTerminator {
    #[inline]
    fn default() -> Self {
        LineTerminator::native()
    }
}

impl fmt::Display for LineTerminator {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
