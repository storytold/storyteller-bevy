//! Errors which may occur when manipulating `Bvh` files.

use crate::Axis;
use lexical::Error as LexicalError;
use std::{error::Error as StdError, fmt, io};

/// Errors which may arise when loading a `Bvh` file from
/// a `Reader`.
#[derive(Debug)]
pub struct LoadError {
    /// The error kind.
    kind: LoadErrorKind,
}

impl LoadError {
    /// Get the line where the error occurred, or `None` if there is
    /// no associated line number.
    #[inline]
    pub fn line(&self) -> Option<usize> {
        match self.kind {
            LoadErrorKind::Joints(ref e) => e.line(),
            LoadErrorKind::Motion(ref e) => e.line(),
        }
    }

    /// Returns the `LoadError` kind.
    #[inline]
    pub fn kind(&self) -> &LoadErrorKind {
        &self.kind
    }

    /// Unwraps the `LoadErrorKind` from the `LoadError`.
    #[inline]
    pub fn into_kind(self) -> LoadErrorKind {
        self.kind
    }
}

impl<K: Into<LoadErrorKind>> From<K> for LoadError {
    #[inline]
    fn from(kind: K) -> Self {
        LoadError { kind: kind.into() }
    }
}

impl fmt::Display for LoadError {
    #[inline]
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = match self.kind {
            LoadErrorKind::Joints(_) => "Could not load hierarchy",
            LoadErrorKind::Motion(_) => "Could not load motion",
        };

        write!(fmtr, "{}: {}", desc, self.source().unwrap())
    }
}

impl StdError for LoadError {
    #[inline]
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self.kind {
            LoadErrorKind::Joints(ref e) => Some(e),
            LoadErrorKind::Motion(ref e) => Some(e),
        }
    }
}

/// The kind of the `LoadError`.
#[derive(Debug)]
pub enum LoadErrorKind {
    /// An error occurred when loading the joints hierarchy.
    Joints(LoadJointsError),
    /// An error occurred when loading the motion values.
    Motion(LoadMotionError),
}

impl From<LoadJointsError> for LoadErrorKind {
    #[inline]
    fn from(e: LoadJointsError) -> Self {
        LoadErrorKind::Joints(e)
    }
}

impl From<LoadMotionError> for LoadErrorKind {
    #[inline]
    fn from(e: LoadMotionError) -> Self {
        LoadErrorKind::Motion(e)
    }
}

/// Represents an error which occurred when loading the `Joints` of the
/// bvh file.
#[derive(Debug)]
pub enum LoadJointsError {
    /// An I/O error occurred.
    Io(io::Error),
    /// The skeletal hierarchy is missing the `Root` joint.
    MissingRoot,
    /// A name could not be found for the `Joint`.
    MissingJointName {
        /// Line number in the source bvh where the error occurred.
        line: usize,
    },
    /// A `CHANNELS` section was encountered in the wrong location.
    UnexpectedChannelsSection {
        /// Line number in the source bvh where the error occurred.
        line: usize,
    },
    /// The number of channels could not be parsed in a `CHANNELS` section.
    ParseNumChannelsError {
        /// The parse error, if there was a malformed string to parse.
        error: Option<LexicalError>,
        /// Line number in the source bvh where the error occurred.
        line: usize,
    },
    /// A channel type could not be parsed in the `CHANNELS` section.
    ParseChannelError {
        /// The parse error.
        error: ParseChannelError,
        /// Line number in the source bvh where the error occurred.
        line: usize,
    },
    /// An `OFFSET` section was encountered in the wrong location.
    UnexpectedOffsetSection {
        /// Line number in the source bvh where the error occurred.
        line: usize,
    },
    /// An axis in the `OFFSET` section could not be parsed into a value.
    ParseOffsetError {
        /// The parse error.
        parse_float_error: LexicalError,
        /// The axis of the offset which could not be parsed.
        axis: Axis,
        /// Line number in the source bvh where the error occurred.
        line: usize,
    },
    /// An `OFFSET` section was missing an axis in the offset vector.
    MissingOffsetAxis {
        /// The smallest axis which was missing.
        axis: Axis,
        /// Line number in the source bvh where the error occurred.
        line: usize,
    },
}

impl LoadJointsError {
    /// Get the line where the error occurred, or `None` if there is
    /// no associated line number.
    #[inline]
    pub fn line(&self) -> Option<usize> {
        match *self {
            LoadJointsError::MissingJointName { line }
            | LoadJointsError::UnexpectedChannelsSection { line }
            | LoadJointsError::ParseNumChannelsError { line, .. }
            | LoadJointsError::ParseChannelError { line, .. }
            | LoadJointsError::UnexpectedOffsetSection { line }
            | LoadJointsError::ParseOffsetError { line, .. }
            | LoadJointsError::MissingOffsetAxis { line, .. } => Some(line),
            _ => None,
        }
    }
}

impl From<io::Error> for LoadJointsError {
    #[inline]
    fn from(e: io::Error) -> Self {
        LoadJointsError::Io(e)
    }
}

impl fmt::Display for LoadJointsError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            LoadJointsError::Io(ref e) => fmt::Display::fmt(&e, f),
            LoadJointsError::MissingRoot => f.write_str("The root heirarchy could not be found"),
            LoadJointsError::MissingJointName { line } => {
                write!(f, "{}: the name is missing from the joints section", line)
            }
            LoadJointsError::UnexpectedChannelsSection { line } => write!(
                f,
                "{}: unexpectedly encountered a \"CHANNELS\" section",
                line
            ),
            LoadJointsError::ParseNumChannelsError { line, .. } => {
                write!(f, "{}: could not find the number of channels", line)
            }
            LoadJointsError::ParseChannelError { line, .. } => {
                write!(f, "{}: could not parse channel", line)
            }
            LoadJointsError::UnexpectedOffsetSection { line } => write!(
                f,
                "{}: unexpectedly encountered an \"OFFSET\" section",
                line
            ),
            LoadJointsError::ParseOffsetError { axis, line, .. } => {
                write!(f, "{}: could not parse the {}-axis offset", line, axis,)
            }
            LoadJointsError::MissingOffsetAxis { axis, line } => {
                write!(f, "{}: the {}-axis offset value is missing", line, axis)
            }
        }
    }
}

impl StdError for LoadJointsError {
    #[inline]
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            LoadJointsError::Io(ref e) => Some(e),
            /*
            LoadJointsError::ParseNumChannelsError { ref error, .. } => {
                error.as_ref().map(|e| e as &(dyn StdError + 'static))
            }
            */
            LoadJointsError::ParseChannelError { ref error, .. } => Some(error),
            /*
            LoadJointsError::ParseOffsetError {
                ref parse_float_error,
                ..
            } => Some(parse_float_error),
            */
            _ => None,
        }
    }
}

/// Represents an error which occurred when loading the motion of the
/// bvh file.
#[derive(Debug)]
pub enum LoadMotionError {
    /// An I/O error occurred.
    Io(io::Error),
    /// The `MOTION` section is missing in the bvh.
    MissingMotionSection {
        /// The line where the error occurred.
        line: usize,
    },
    /// The "Number of Frames" section could not be parsed in the bvh.
    MissingNumFrames {
        /// The parse error, or `None` if there was no number to be parsed.
        parse_error: Option<LexicalError>,
        /// The line where the error occurred.
        line: usize,
    },
    /// The "Frame Time" section could not be parsed in the bvh.
    MissingFrameTime {
        /// The parse error, or `None` if there was no number to be parsed.
        parse_error: Option<LexicalError>,
        /// The line where the error occurred.
        line: usize,
    },
    /// The motion values section could not be parsed in the bvh.
    ParseMotionSection {
        /// The parse error, or `None` if there was no number to be parsed.
        parse_error: LexicalError,
        /// The index of the motion value where the error occurred.
        channel_index: usize,
        /// The line where the error occurred.
        line: usize,
    },
    /// There was a discrepancy between the number of motion values promised
    /// by the file and the actual amount.
    MotionCountMismatch {
        /// Actual number of motion values parsed.
        actual_total_motion_values: usize,
        /// Expected number of motion values.
        expected_total_motion_values: usize,
        /// Expected number of frames.
        expected_num_frames: usize,
        /// Expected number of clips.
        expected_num_clips: usize,
    },
}

impl LoadMotionError {
    /// Get the line where the error occurred, or `None` if there is
    /// no associated line number.
    pub fn line(&self) -> Option<usize> {
        match *self {
            LoadMotionError::MissingMotionSection { line }
            | LoadMotionError::MissingNumFrames { line, .. }
            | LoadMotionError::MissingFrameTime { line, .. }
            | LoadMotionError::ParseMotionSection { line, .. } => Some(line),
            _ => None,
        }
    }
}

impl fmt::Display for LoadMotionError {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            LoadMotionError::Io(ref e) => fmt::Display::fmt(e, fmtr),
            LoadMotionError::MissingMotionSection { line } => {
                write!(
                    fmtr,
                    "{}: The 'MOTION' section of the bvh file is missing",
                    line
                )
            }
            LoadMotionError::MissingNumFrames {
                ref parse_error,
                line,
            } => {
                if let Some(_) = parse_error {
                    write!(fmtr, "{}: could not parse the num frames value", line)
                } else {
                    write!(
                        fmtr,
                        "{}: The number of frames section is missing from the bvh file",
                        line
                    )
                }
            }
            LoadMotionError::MissingFrameTime {
                ref parse_error,
                line,
            } => {
                if let Some(_) = parse_error {
                    write!(fmtr, "{}: could not parse the frame time", line)
                } else {
                    write!(fmtr, "{}: Could not parse the frame time", line)
                }
            }
            LoadMotionError::ParseMotionSection { line, .. } => {
                write!(fmtr, "{}: Could not parse the motion value", line)
            }
            LoadMotionError::MotionCountMismatch {
                actual_total_motion_values,
                expected_total_motion_values,
                expected_num_frames,
                expected_num_clips,
            } => {
                write!(
                    fmtr,
                    "expected to find {} motion values, found {} values (num frames = {}, num clips = {})",
                    expected_total_motion_values,
                    actual_total_motion_values,
                    expected_num_frames,
                    expected_num_clips)
            }
        }
    }
}

impl StdError for LoadMotionError {
    #[inline]
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            LoadMotionError::Io(ref e) => Some(e),
            /*
            LoadMotionError::MissingFrameTime {
                parse_error: Some(ref e),
                ..
            } => Some(e),
            LoadMotionError::ParseMotionSection {
                ref parse_error, ..
            } => Some(parse_error),
            LoadMotionError::MissingNumFrames {
                parse_error: Some(ref e),
                ..
            } => Some(e),
            */
            _ => None,
        }
    }
}

impl From<io::Error> for LoadMotionError {
    #[inline]
    fn from(e: io::Error) -> Self {
        LoadMotionError::Io(e)
    }
}

/// An error which may occurr when setting a motion which is out
/// of bounds.
#[derive(Clone, Debug, PartialEq)]
pub enum SetMotionError {
    /// The frame was out of bounds.
    BadFrame(usize),
    /// The channel was out of bounds.
    BadChannel(usize),
}

impl fmt::Display for SetMotionError {
    #[inline]
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            SetMotionError::BadFrame(frame) => write!(fmtr, "Frame {} was out of bounds", frame),
            SetMotionError::BadChannel(channel) => {
                write!(fmtr, "Channel {} of the bvh was out of bounds", channel,)
            }
        }
    }
}

impl StdError for SetMotionError {
    #[inline]
    fn description(&self) -> &'static str {
        match *self {
            SetMotionError::BadFrame(_) => "The frame was out of bounds",
            SetMotionError::BadChannel(_) => "The channel was out of bounds",
        }
    }
}

/// Represents an error which may occur when attempting to parse a
/// `BString` into a `ChannelType`.
#[derive(Debug)]
pub struct ParseChannelError {
    // @TODO(burtonageo): Borrow the erroneous string when hrts
    // land.
    bad_string: Vec<u8>,
}

impl ParseChannelError {
    /// Get the `BString` which caused the parse error.
    #[inline]
    pub fn into_inner(self) -> Vec<u8> {
        self.bad_string
    }
}

impl<S: Into<Vec<u8>>> From<S> for ParseChannelError {
    #[inline]
    fn from(bad_string: S) -> Self {
        ParseChannelError {
            bad_string: bad_string.into(),
        }
    }
}

impl fmt::Display for ParseChannelError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "The channel could not be parsed from the given string: {:?}",
            &self.bad_string
        )
    }
}

impl StdError for ParseChannelError {}

/// An error which may occur when attempting to insert a frame into a
/// `Bvh`.
#[derive(Debug)]
pub struct FrameInsertError {
    kind: FrameInsertErrorKind,
}

impl FrameInsertError {
    pub(crate) const fn incorrect_len(expected: usize, actual: usize) -> Self {
        Self {
            kind: FrameInsertErrorKind::IncorrectFrameLength { expected, actual },
        }
    }
}

impl fmt::Display for FrameInsertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            FrameInsertErrorKind::IncorrectFrameLength { expected, actual } => {
                write!(
                    f,
                    "Attempted to insert a frame with {} channels into a bvh containing {} channels",
                    actual,
                    expected,
                )
            }
        }
    }
}

impl StdError for FrameInsertError {}

#[derive(Debug)]
enum FrameInsertErrorKind {
    IncorrectFrameLength { expected: usize, actual: usize },
}

/// An error which may occur when attempting to remove a frame from a
/// `Bvh`.
#[derive(Debug)]
pub struct FrameRemoveError {
    index: usize,
}

impl FrameRemoveError {
    pub(crate) const fn new(index: usize) -> Self {
        Self { index }
    }
}

impl fmt::Display for FrameRemoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Attempted to remove a frame at index {} which does not exist",
            self.index
        )
    }
}

impl StdError for FrameRemoveError {}
