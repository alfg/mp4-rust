use std::fmt;
use std::convert::TryInto;

pub use bytes::Bytes;

mod error;
pub use error::Error;

mod atoms;

mod reader;
pub use reader::Mp4Reader;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum TrackType {
    Audio,
    Video,
    Metadata,
    Unknown,
}

#[derive(Debug)]
pub struct Mp4Sample {
    pub start_time: u64,
    pub duration: u32,
    pub rendering_offset: i32,
    pub is_sync: bool,
    pub bytes: Bytes,
}

impl PartialEq for Mp4Sample {
    fn eq(&self, other: &Self) -> bool {
        self.start_time == other.start_time
            && self.duration == other.duration
            && self.rendering_offset == other.rendering_offset
            && self.is_sync == other.is_sync
            && self.bytes.len() == other.bytes.len() // XXX for easy check
    }
}

impl fmt::Display for Mp4Sample {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "start_time {}, duration {}, rendering_offset {}, is_sync {}, length {}",
               self.start_time, self.duration, self.rendering_offset, self.is_sync,
               self.bytes.len())
    }
}
