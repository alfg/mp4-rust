use std::convert::TryFrom;
use std::fmt;

use crate::atoms::*;
use crate::*;

pub use bytes::Bytes;

impl fmt::Debug for BoxType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fourcc: FourCC = From::from(self.clone());
        write!(f, "{}", fourcc)
    }
}

impl fmt::Display for BoxType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fourcc: FourCC = From::from(self.clone());
        write!(f, "{}", fourcc)
    }
}

#[derive(Default, PartialEq, Clone)]
pub struct FourCC {
    pub value: String,
}

impl From<u32> for FourCC {
    fn from(number: u32) -> Self {
        let mut box_chars = Vec::new();
        for x in 0..4 {
            let c = (number >> (x * 8) & 0x0000_00FF) as u8;
            box_chars.push(c);
        }
        box_chars.reverse();

        let box_string = match String::from_utf8(box_chars) {
            Ok(t) => t,
            _ => String::from("null"), // error to retrieve fourcc
        };

        FourCC { value: box_string }
    }
}

impl From<FourCC> for u32 {
    fn from(fourcc: FourCC) -> u32 {
        (&fourcc).into()
    }
}

impl From<&FourCC> for u32 {
    fn from(fourcc: &FourCC) -> u32 {
        let mut b: [u8; 4] = Default::default();
        b.copy_from_slice(fourcc.value.as_bytes());
        u32::from_be_bytes(b)
    }
}

impl From<String> for FourCC {
    fn from(fourcc: String) -> FourCC {
        let value = if fourcc.len() > 4 {
            fourcc[0..4].to_string()
        } else {
            fourcc
        };
        FourCC { value }
    }
}

impl From<&str> for FourCC {
    fn from(fourcc: &str) -> FourCC {
        let value = if fourcc.len() > 4 {
            fourcc[0..4].to_string()
        } else {
            fourcc.to_string()
        };
        FourCC { value }
    }
}

impl From<BoxType> for FourCC {
    fn from(t: BoxType) -> FourCC {
        let box_num: u32 = Into::into(t);
        From::from(box_num)
    }
}

impl fmt::Debug for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let code: u32 = self.into();
        write!(f, "{} / {:#010X}", self.value, code)
    }
}

impl fmt::Display for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

const HANDLER_TYPE_VIDEO: &str = "vide";
const HANDLER_TYPE_AUDIO: &str = "soun";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrackType {
    Video,
    Audio,
}

impl fmt::Display for TrackType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s: &str = self.into();
        write!(f, "{}", s)
    }
}

impl TryFrom<&str> for TrackType {
    type Error = Error;
    fn try_from(handler: &str) -> Result<TrackType> {
        match handler {
            HANDLER_TYPE_VIDEO => Ok(TrackType::Video),
            HANDLER_TYPE_AUDIO => Ok(TrackType::Audio),
            _ => Err(Error::InvalidData("unsupported handler type")),
        }
    }
}

impl Into<&str> for TrackType {
    fn into(self) -> &'static str {
        match self {
            TrackType::Video => HANDLER_TYPE_VIDEO,
            TrackType::Audio => HANDLER_TYPE_AUDIO,
        }
    }
}

impl Into<&str> for &TrackType {
    fn into(self) -> &'static str {
        match self {
            TrackType::Video => HANDLER_TYPE_VIDEO,
            TrackType::Audio => HANDLER_TYPE_AUDIO,
        }
    }
}

impl TryFrom<&FourCC> for TrackType {
    type Error = Error;
    fn try_from(fourcc: &FourCC) -> Result<TrackType> {
        TrackType::try_from(fourcc.value.as_str())
    }
}

impl Into<FourCC> for TrackType {
    fn into(self) -> FourCC {
        let s: &str = self.into();
        FourCC::from(s)
    }
}

const MEDIA_TYPE_H264: &str = "h264";
const MEDIA_TYPE_AAC: &str = "aac";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MediaType {
    H264,
    AAC,
}

impl fmt::Display for MediaType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s: &str = self.into();
        write!(f, "{}", s)
    }
}

impl TryFrom<&str> for MediaType {
    type Error = Error;
    fn try_from(media: &str) -> Result<MediaType> {
        match media {
            MEDIA_TYPE_H264 => Ok(MediaType::H264),
            MEDIA_TYPE_AAC => Ok(MediaType::AAC),
            _ => Err(Error::InvalidData("unsupported media type")),
        }
    }
}

impl Into<&str> for MediaType {
    fn into(self) -> &'static str {
        match self {
            MediaType::H264 => MEDIA_TYPE_H264,
            MediaType::AAC => MEDIA_TYPE_AAC,
        }
    }
}

impl Into<&str> for &MediaType {
    fn into(self) -> &'static str {
        match self {
            MediaType::H264 => MEDIA_TYPE_H264,
            MediaType::AAC => MEDIA_TYPE_AAC,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SampleFreqIndex {
    Freq96000 = 0x0,
    Freq88200 = 0x1,
    Freq64000 = 0x2,
    Freq48000 = 0x3,
    Freq44100 = 0x4,
    Freq32000 = 0x5,
    Freq24000 = 0x6,
    Freq22050 = 0x7,
    Freq16000 = 0x8,
    Freq12000 = 0x9,
    Freq11025 = 0xa,
    Freq8000 = 0xb,
}

impl TryFrom<u8> for SampleFreqIndex {
    type Error = Error;
    fn try_from(value: u8) -> Result<SampleFreqIndex> {
        match value {
            0x0 => Ok(SampleFreqIndex::Freq96000),
            0x1 => Ok(SampleFreqIndex::Freq88200),
            0x2 => Ok(SampleFreqIndex::Freq64000),
            0x3 => Ok(SampleFreqIndex::Freq48000),
            0x4 => Ok(SampleFreqIndex::Freq44100),
            0x5 => Ok(SampleFreqIndex::Freq32000),
            0x6 => Ok(SampleFreqIndex::Freq24000),
            0x7 => Ok(SampleFreqIndex::Freq22050),
            0x8 => Ok(SampleFreqIndex::Freq16000),
            0x9 => Ok(SampleFreqIndex::Freq12000),
            0xa => Ok(SampleFreqIndex::Freq11025),
            0xb => Ok(SampleFreqIndex::Freq8000),
            _ => Err(Error::InvalidData("invalid sampling frequency index")),
        }
    }
}

impl SampleFreqIndex {
    pub fn freq(&self) -> u32 {
        match self {
            &SampleFreqIndex::Freq96000 => 96000,
            &SampleFreqIndex::Freq88200 => 88200,
            &SampleFreqIndex::Freq64000 => 64000,
            &SampleFreqIndex::Freq48000 => 48000,
            &SampleFreqIndex::Freq44100 => 44100,
            &SampleFreqIndex::Freq32000 => 32000,
            &SampleFreqIndex::Freq24000 => 24000,
            &SampleFreqIndex::Freq22050 => 22050,
            &SampleFreqIndex::Freq16000 => 16000,
            &SampleFreqIndex::Freq12000 => 12000,
            &SampleFreqIndex::Freq11025 => 11025,
            &SampleFreqIndex::Freq8000 => 8000,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ChannelConfig {
    Mono = 0x1,
    Stereo = 0x2,
    Three = 0x3,
    Four = 0x4,
    Five = 0x5,
    FiveOne = 0x6,
    SevenOne = 0x7,
}

impl TryFrom<u8> for ChannelConfig {
    type Error = Error;
    fn try_from(value: u8) -> Result<ChannelConfig> {
        match value {
            0x1 => Ok(ChannelConfig::Mono),
            0x2 => Ok(ChannelConfig::Stereo),
            0x3 => Ok(ChannelConfig::Three),
            0x4 => Ok(ChannelConfig::Four),
            0x5 => Ok(ChannelConfig::Five),
            0x6 => Ok(ChannelConfig::FiveOne),
            0x7 => Ok(ChannelConfig::SevenOne),
            _ => Err(Error::InvalidData("invalid channel configuration")),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MediaConfig {
    AVC {
        width: u16,
        height: u16,
        // sps: Vec<u8>,
        // pps: Vec<u8>,
    },
    AAC {
        freq_index: SampleFreqIndex,
        chan_conf: ChannelConfig,
    },
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
        write!(
            f,
            "start_time {}, duration {}, rendering_offset {}, is_sync {}, length {}",
            self.start_time,
            self.duration,
            self.rendering_offset,
            self.is_sync,
            self.bytes.len()
        )
    }
}

pub fn creation_time(creation_time: u64) -> u64 {
    // convert from MP4 epoch (1904-01-01) to Unix epoch (1970-01-01)
    if creation_time >= 2082844800 {
        creation_time - 2082844800
    } else {
        creation_time
    }
}
