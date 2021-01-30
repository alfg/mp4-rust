use std::convert::TryFrom;
use std::fmt;
use serde::{Serialize};

use crate::mp4box::*;
use crate::*;

pub use bytes::Bytes;
pub use num_rational::Ratio;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct FixedPointU8(Ratio<u16>);

impl FixedPointU8 {
    pub fn new(val: u8) -> Self {
        Self(Ratio::new_raw(val as u16 * 0x100, 0x100))
    }

    pub fn new_raw(val: u16) -> Self {
        Self(Ratio::new_raw(val, 0x100))
    }

    pub fn value(&self) -> u8 {
        self.0.to_integer() as u8
    }

    pub fn raw_value(&self) -> u16 {
        *self.0.numer()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct FixedPointI8(Ratio<i16>);

impl FixedPointI8 {
    pub fn new(val: i8) -> Self {
        Self(Ratio::new_raw(val as i16 * 0x100, 0x100))
    }

    pub fn new_raw(val: i16) -> Self {
        Self(Ratio::new_raw(val, 0x100))
    }

    pub fn value(&self) -> i8 {
        self.0.to_integer() as i8
    }

    pub fn raw_value(&self) -> i16 {
        *self.0.numer()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct FixedPointU16(Ratio<u32>);

impl FixedPointU16 {
    pub fn new(val: u16) -> Self {
        Self(Ratio::new_raw(val as u32 * 0x10000, 0x10000))
    }

    pub fn new_raw(val: u32) -> Self {
        Self(Ratio::new_raw(val, 0x10000))
    }

    pub fn value(&self) -> u16 {
        self.0.to_integer() as u16
    }

    pub fn raw_value(&self) -> u32 {
        *self.0.numer()
    }
}

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

#[derive(Default, PartialEq, Clone, Copy, Serialize)]
pub struct FourCC {
    pub value: [u8; 4],
}

impl std::str::FromStr for FourCC {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if let [a, b, c, d] = s.as_bytes() {
            Ok(Self { value: [*a, *b, *c, *d] })
        } else {
            Err(Error::InvalidData("expected exactly four bytes in string"))
        }
    }
}

impl From<u32> for FourCC {
    fn from(number: u32) -> Self {
        FourCC { value: number.to_be_bytes() }
    }
}

impl From<FourCC> for u32 {
    fn from(fourcc: FourCC) -> u32 {
        (&fourcc).into()
    }
}

impl From<&FourCC> for u32 {
    fn from(fourcc: &FourCC) -> u32 {
        u32::from_be_bytes(fourcc.value)
    }
}

impl From<[u8; 4]> for FourCC {
    fn from(value: [u8; 4]) -> FourCC {
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
        let string = String::from_utf8_lossy(&self.value[..]);
        write!(f, "{} / {:#010X}", string, code)
    }
}

impl fmt::Display for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.value[..]))
    }
}

const DISPLAY_TYPE_VIDEO: &str = "Video";
const DISPLAY_TYPE_AUDIO: &str = "Audio";
const DISPLAY_TYPE_SUBTITLE: &str = "Subtitle";

const HANDLER_TYPE_VIDEO: &str = "vide";
const HANDLER_TYPE_VIDEO_FOURCC: [u8; 4] = [b'v', b'i', b'd', b'e'];

const HANDLER_TYPE_AUDIO: &str = "soun";
const HANDLER_TYPE_AUDIO_FOURCC: [u8; 4] = [b's', b'o', b'u', b'n'];

const HANDLER_TYPE_SUBTITLE: &str = "sbtl";
const HANDLER_TYPE_SUBTITLE_FOURCC: [u8; 4] = [b's', b'b', b't', b'l'];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrackType {
    Video,
    Audio,
    Subtitle,
}

impl fmt::Display for TrackType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            TrackType::Video => DISPLAY_TYPE_VIDEO,
            TrackType::Audio => DISPLAY_TYPE_AUDIO,
            TrackType::Subtitle => DISPLAY_TYPE_SUBTITLE,
        };
        write!(f, "{}", s)
    }
}

impl TryFrom<&str> for TrackType {
    type Error = Error;
    fn try_from(handler: &str) -> Result<TrackType> {
        match handler {
            HANDLER_TYPE_VIDEO => Ok(TrackType::Video),
            HANDLER_TYPE_AUDIO => Ok(TrackType::Audio),
            HANDLER_TYPE_SUBTITLE => Ok(TrackType::Subtitle),
            _ => Err(Error::InvalidData("unsupported handler type")),
        }
    }
}

impl TryFrom<&FourCC> for TrackType {
    type Error = Error;
    fn try_from(fourcc: &FourCC) -> Result<TrackType> {
        match fourcc.value {
            HANDLER_TYPE_VIDEO_FOURCC => Ok(TrackType::Video),
            HANDLER_TYPE_AUDIO_FOURCC => Ok(TrackType::Audio),
            HANDLER_TYPE_SUBTITLE_FOURCC => Ok(TrackType::Subtitle),
            _ => Err(Error::InvalidData("unsupported handler type")),
        }
    }
}

impl Into<FourCC> for TrackType {
    fn into(self) -> FourCC {
        match self {
            TrackType::Video => HANDLER_TYPE_VIDEO_FOURCC.into(),
            TrackType::Audio => HANDLER_TYPE_AUDIO_FOURCC.into(),
            TrackType::Subtitle => HANDLER_TYPE_SUBTITLE_FOURCC.into(),
        }
    }
}

const MEDIA_TYPE_H264: &str = "h264";
const MEDIA_TYPE_H265: &str = "h265";
const MEDIA_TYPE_VP9: &str = "vp9";
const MEDIA_TYPE_AAC: &str = "aac";
const MEDIA_TYPE_TTXT: &str = "ttxt";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MediaType {
    H264,
    H265,
    VP9,
    AAC,
    TTXT,
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
            MEDIA_TYPE_H265 => Ok(MediaType::H265),
            MEDIA_TYPE_VP9 => Ok(MediaType::VP9),
            MEDIA_TYPE_AAC => Ok(MediaType::AAC),
            MEDIA_TYPE_TTXT => Ok(MediaType::TTXT),
            _ => Err(Error::InvalidData("unsupported media type")),
        }
    }
}

impl Into<&str> for MediaType {
    fn into(self) -> &'static str {
        match self {
            MediaType::H264 => MEDIA_TYPE_H264,
            MediaType::H265 => MEDIA_TYPE_H265,
            MediaType::VP9 => MEDIA_TYPE_VP9,
            MediaType::AAC => MEDIA_TYPE_AAC,
            MediaType::TTXT => MEDIA_TYPE_TTXT,
        }
    }
}

impl Into<&str> for &MediaType {
    fn into(self) -> &'static str {
        match self {
            MediaType::H264 => MEDIA_TYPE_H264,
            MediaType::H265 => MEDIA_TYPE_H265,
            MediaType::VP9 => MEDIA_TYPE_VP9,
            MediaType::AAC => MEDIA_TYPE_AAC,
            MediaType::TTXT => MEDIA_TYPE_TTXT,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AvcProfile {
    AvcConstrainedBaseline, // 66 with constraint set 1
    AvcBaseline,            // 66,
    AvcMain,                // 77,
    AvcExtended,            // 88,
    AvcHigh,                // 100
                            // TODO Progressive High Profile, Constrained High Profile, ...
}

impl TryFrom<(u8, u8)> for AvcProfile {
    type Error = Error;
    fn try_from(value: (u8, u8)) -> Result<AvcProfile> {
        let profile = value.0;
        let constraint_set1_flag = value.1 & 0x40 >> 7;
        match (profile, constraint_set1_flag) {
            (66, 1) => Ok(AvcProfile::AvcConstrainedBaseline),
            (66, 0) => Ok(AvcProfile::AvcBaseline),
            (77, _) => Ok(AvcProfile::AvcMain),
            (88, _) => Ok(AvcProfile::AvcExtended),
            (100, _) => Ok(AvcProfile::AvcHigh),
            _ => Err(Error::InvalidData("unsupported avc profile")),
        }
    }
}

impl fmt::Display for AvcProfile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let profile = match self {
            AvcProfile::AvcConstrainedBaseline => "Constrained Baseline",
            AvcProfile::AvcBaseline => "Baseline",
            AvcProfile::AvcMain => "Main",
            AvcProfile::AvcExtended => "Extended",
            AvcProfile::AvcHigh => "High",
        };
        write!(f, "{}", profile)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AudioObjectType {
    AacMain = 1,
    AacLowComplexity = 2,
    AacScalableSampleRate = 3,
    AacLongTermPrediction = 4,
}

impl TryFrom<u8> for AudioObjectType {
    type Error = Error;
    fn try_from(value: u8) -> Result<AudioObjectType> {
        match value {
            1 => Ok(AudioObjectType::AacMain),
            2 => Ok(AudioObjectType::AacLowComplexity),
            3 => Ok(AudioObjectType::AacScalableSampleRate),
            4 => Ok(AudioObjectType::AacLongTermPrediction),
            _ => Err(Error::InvalidData("invalid audio object type")),
        }
    }
}

impl fmt::Display for AudioObjectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let type_str = match self {
            AudioObjectType::AacMain => "main",
            AudioObjectType::AacLowComplexity => "LC",
            AudioObjectType::AacScalableSampleRate => "SSR",
            AudioObjectType::AacLongTermPrediction => "LTP",
        };
        write!(f, "{}", type_str)
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
    Freq7350 = 0xc,
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
            0xc => Ok(SampleFreqIndex::Freq7350),
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
            &SampleFreqIndex::Freq7350 => 7350,
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

impl fmt::Display for ChannelConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            ChannelConfig::Mono => "mono",
            ChannelConfig::Stereo => "stereo",
            ChannelConfig::Three => "three",
            ChannelConfig::Four => "four",
            ChannelConfig::Five => "five",
            ChannelConfig::FiveOne => "five.one",
            ChannelConfig::SevenOne => "seven.one",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct AvcConfig {
    pub width: u16,
    pub height: u16,
    pub seq_param_set: Vec<u8>,
    pub pic_param_set: Vec<u8>,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct HevcConfig {
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Vp9Config {
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AacConfig {
    pub bitrate: u32,
    pub profile: AudioObjectType,
    pub freq_index: SampleFreqIndex,
    pub chan_conf: ChannelConfig,
}

impl Default for AacConfig {
    fn default() -> Self {
        Self {
            bitrate: 0,
            profile: AudioObjectType::AacLowComplexity,
            freq_index: SampleFreqIndex::Freq48000,
            chan_conf: ChannelConfig::Stereo,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct TtxtConfig {}

#[derive(Debug, PartialEq, Clone)]
pub enum MediaConfig {
    AvcConfig(AvcConfig),
    HevcConfig(HevcConfig),
    Vp9Config(Vp9Config),
    AacConfig(AacConfig),
    TtxtConfig(TtxtConfig),
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
