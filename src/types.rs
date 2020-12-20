#[cfg(feature = "use_serde")]
use serde::Serialize;
use std::convert::TryFrom;
use std::fmt;
use std::marker::PhantomData;

use crate::mp4box::*;
use crate::*;

pub use bytes::Bytes;

pub trait FixedPointKind {
    const POINT: usize;
    type Carrier;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FPi4_4 {}
impl FixedPointKind for FPi4_4 {
    const POINT: usize = 4;
    type Carrier = i8;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FPu4_4 {}
impl FixedPointKind for FPu4_4 {
    const POINT: usize = 4;
    type Carrier = u8;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FPi8_8 {}
impl FixedPointKind for FPi8_8 {
    const POINT: usize = 8;
    type Carrier = i16;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FPu8_8 {}
impl FixedPointKind for FPu8_8 {
    const POINT: usize = 8;
    type Carrier = u16;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FPi16_16 {}
impl FixedPointKind for FPi16_16 {
    const POINT: usize = 16;
    type Carrier = i32;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FPu16_16 {}
impl FixedPointKind for FPu16_16 {
    const POINT: usize = 16;
    type Carrier = u32;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FPi2_30 {}
impl FixedPointKind for FPi2_30 {
    const POINT: usize = 30;
    type Carrier = i32;
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "use_serde", derive(Serialize))]
#[cfg_attr(feature = "use_serde", serde(transparent))]
pub struct FixedPoint<T: FixedPointKind>(T::Carrier, PhantomData<T>);

impl<T: FixedPointKind> fmt::Debug for FixedPoint<T>
where
    T::Carrier: Into<f64> + Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let raw_float: f64 = self.0.into();
        let float = raw_float / (1 << T::POINT) as f64;
        write!(
            f,
            "{}fp{}.{}",
            float,
            std::any::type_name::<T::Carrier>(),
            T::POINT
        )
    }
}

impl<T: FixedPointKind> FixedPoint<T> {
    pub fn new_whole(val: T::Carrier) -> Self
    where
        T::Carrier: std::ops::Shl<Output = T::Carrier> + TryFrom<usize>,
    {
        let point: T::Carrier = TryFrom::try_from(T::POINT).map_err(|_| ()).unwrap();
        Self(val << point, PhantomData)
    }
    pub fn new_raw(val: T::Carrier) -> Self {
        Self(val, PhantomData)
    }
}

impl<T: FixedPointKind> FixedPoint<T>
where
    T::Carrier: Copy,
{
    pub fn value(&self) -> T::Carrier {
        self.0
    }
    pub fn raw_value(&self) -> T::Carrier {
        self.0
    }
}

impl<T: FixedPointKind> Default for FixedPoint<T>
where
    T::Carrier: Default,
{
    fn default() -> Self {
        Self(Default::default(), PhantomData)
    }
}

pub type FixedPointU8 = FixedPoint<FPu4_4>;
pub type FixedPointI8 = FixedPoint<FPi4_4>;
pub type FixedPointU16 = FixedPoint<FPu8_8>;
pub type FixedPointI16 = FixedPoint<FPi8_8>;
pub type FixedPointU32 = FixedPoint<FPu16_16>;
pub type FixedPointI32 = FixedPoint<FPi16_16>;
pub type FixedPointI2_30 = FixedPoint<FPi2_30>;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "use_serde", derive(Serialize))]
pub struct Matrix {
    pub a: FixedPointI32,
    pub b: FixedPointI32,
    pub u: FixedPointI2_30,
    pub c: FixedPointI32,
    pub d: FixedPointI32,
    pub v: FixedPointI2_30,
    pub x: FixedPointI32,
    pub y: FixedPointI32,
    pub w: FixedPointI2_30,
}

impl Default for Matrix {
    fn default() -> Self {
        Matrix {
            a: FixedPointI32::new_whole(1),
            b: FixedPointI32::new_whole(0),
            u: FixedPointI2_30::new_whole(0),
            c: FixedPointI32::new_whole(0),
            d: FixedPointI32::new_whole(1),
            v: FixedPointI2_30::new_whole(0),
            x: FixedPointI32::new_whole(0),
            y: FixedPointI32::new_whole(0),
            w: FixedPointI2_30::new_whole(1),
        }
    }
}

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

impl Matrix {
    pub fn read_from<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        Ok(Matrix {
            a: FixedPointI32::new_raw(reader.read_i32::<BigEndian>()?),
            b: FixedPointI32::new_raw(reader.read_i32::<BigEndian>()?),
            u: FixedPointI2_30::new_raw(reader.read_i32::<BigEndian>()?),
            c: FixedPointI32::new_raw(reader.read_i32::<BigEndian>()?),
            d: FixedPointI32::new_raw(reader.read_i32::<BigEndian>()?),
            v: FixedPointI2_30::new_raw(reader.read_i32::<BigEndian>()?),
            x: FixedPointI32::new_raw(reader.read_i32::<BigEndian>()?),
            y: FixedPointI32::new_raw(reader.read_i32::<BigEndian>()?),
            w: FixedPointI2_30::new_raw(reader.read_i32::<BigEndian>()?),
        })
    }
    pub fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_i32::<BigEndian>(self.a.raw_value())?;
        writer.write_i32::<BigEndian>(self.b.raw_value())?;
        writer.write_i32::<BigEndian>(self.u.raw_value())?;
        writer.write_i32::<BigEndian>(self.c.raw_value())?;
        writer.write_i32::<BigEndian>(self.d.raw_value())?;
        writer.write_i32::<BigEndian>(self.v.raw_value())?;
        writer.write_i32::<BigEndian>(self.x.raw_value())?;
        writer.write_i32::<BigEndian>(self.y.raw_value())?;
        writer.write_i32::<BigEndian>(self.w.raw_value())?;
        Ok(())
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

#[derive(Default, PartialEq, Clone)]
#[cfg_attr(feature = "use_serde", derive(Serialize))]
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

const DISPLAY_TYPE_VIDEO: &str = "Video";
const DISPLAY_TYPE_AUDIO: &str = "Audio";
const DISPLAY_TYPE_SUBTITLE: &str = "Subtitle";

const HANDLER_TYPE_VIDEO: &str = "vide";
const HANDLER_TYPE_AUDIO: &str = "soun";
const HANDLER_TYPE_SUBTITLE: &str = "sbtl";

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

impl Into<&str> for TrackType {
    fn into(self) -> &'static str {
        match self {
            TrackType::Video => HANDLER_TYPE_VIDEO,
            TrackType::Audio => HANDLER_TYPE_AUDIO,
            TrackType::Subtitle => HANDLER_TYPE_SUBTITLE,
        }
    }
}

impl Into<&str> for &TrackType {
    fn into(self) -> &'static str {
        match self {
            TrackType::Video => HANDLER_TYPE_VIDEO,
            TrackType::Audio => HANDLER_TYPE_AUDIO,
            TrackType::Subtitle => HANDLER_TYPE_SUBTITLE,
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
const MEDIA_TYPE_H265: &str = "h265";
const MEDIA_TYPE_AAC: &str = "aac";
const MEDIA_TYPE_TTXT: &str = "ttxt";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MediaType {
    H264,
    H265,
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

#[derive(Debug, PartialEq, Clone)]
pub enum AvcVariant {
    Avc1,
    Avc2,
    Avc3,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AvcConfig {
    pub variant: AvcVariant,
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
