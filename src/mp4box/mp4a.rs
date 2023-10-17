use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Mp4aBox {
    pub data_reference_index: u16,
    pub channelcount: u16,
    pub samplesize: u16,

    #[serde(with = "value_u32")]
    pub samplerate: FixedPointU16,
    pub esds: Option<EsdsBox>,
}

impl Default for Mp4aBox {
    fn default() -> Self {
        Self {
            data_reference_index: 0,
            channelcount: 2,
            samplesize: 16,
            samplerate: FixedPointU16::new(48000),
            esds: Some(EsdsBox::default()),
        }
    }
}

impl Mp4aBox {
    pub fn new(config: &AacConfig) -> Self {
        Self {
            data_reference_index: 1,
            channelcount: config.chan_conf as u16,
            samplesize: 16,
            samplerate: FixedPointU16::new(config.freq_index.freq() as u16),
            esds: Some(EsdsBox::new(config)),
        }
    }

    pub fn get_type(&self) -> BoxType {
        BoxType::Mp4aBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE + 8 + 20;
        if let Some(ref esds) = self.esds {
            size += esds.box_size();
        }
        size
    }
}

impl Mp4Box for Mp4aBox {
    fn box_type(&self) -> BoxType {
        self.get_type()
    }

    fn box_size(&self) -> u64 {
        self.get_size()
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!(
            "channel_count={} sample_size={} sample_rate={}",
            self.channelcount,
            self.samplesize,
            self.samplerate.value()
        );
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for Mp4aBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        reader.read_u32::<BigEndian>()?; // reserved
        reader.read_u16::<BigEndian>()?; // reserved
        let data_reference_index = reader.read_u16::<BigEndian>()?;
        let version = reader.read_u16::<BigEndian>()?;
        reader.read_u16::<BigEndian>()?; // reserved
        reader.read_u32::<BigEndian>()?; // reserved
        let channelcount = reader.read_u16::<BigEndian>()?;
        let samplesize = reader.read_u16::<BigEndian>()?;
        reader.read_u32::<BigEndian>()?; // pre-defined, reserved
        let samplerate = FixedPointU16::new_raw(reader.read_u32::<BigEndian>()?);

        if version == 1 {
            // Skip QTFF
            reader.read_u64::<BigEndian>()?;
            reader.read_u64::<BigEndian>()?;
        }

        // Find esds in mp4a or wave
        let mut esds = None;
        let end = start + size;
        loop {
            let current = reader.stream_position()?;
            if current >= end {
                break;
            }
            let header = BoxHeader::read(reader)?;
            let BoxHeader { name, size: s } = header;
            if s > size {
                return Err(Error::InvalidData(
                    "mp4a box contains a box with a larger size than it",
                ));
            }
            if name == BoxType::EsdsBox {
                esds = Some(EsdsBox::read_box(reader, s)?);
                break;
            } else if name == BoxType::WaveBox {
                // Typically contains frma, mp4a, esds, and a terminator atom
            } else {
                // Skip boxes
                let skip_to = current + s;
                skip_bytes_to(reader, skip_to)?;
            }
        }

        skip_bytes_to(reader, end)?;

        Ok(Mp4aBox {
            data_reference_index,
            channelcount,
            samplesize,
            samplerate,
            esds,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for Mp4aBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        writer.write_u32::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(self.data_reference_index)?;

        writer.write_u64::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(self.channelcount)?;
        writer.write_u16::<BigEndian>(self.samplesize)?;
        writer.write_u32::<BigEndian>(0)?; // reserved
        writer.write_u32::<BigEndian>(self.samplerate.raw_value())?;

        if let Some(ref esds) = self.esds {
            esds.write_box(writer)?;
        }

        Ok(size)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct EsdsBox {
    pub version: u8,
    pub flags: u32,
    pub es_desc: ESDescriptor,
}

impl EsdsBox {
    pub fn new(config: &AacConfig) -> Self {
        Self {
            version: 0,
            flags: 0,
            es_desc: ESDescriptor::new(config),
        }
    }
}

impl Mp4Box for EsdsBox {
    fn box_type(&self) -> BoxType {
        BoxType::EsdsBox
    }

    fn box_size(&self) -> u64 {
        let es_desc_size = self.es_desc.desc_size();
        HEADER_SIZE
            + HEADER_EXT_SIZE
            + 1
            + size_of_length(es_desc_size) as u64
            + es_desc_size as u64
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        Ok(String::new())
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for EsdsBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        let mut es_desc = None;

        let mut current = reader.stream_position()?;
        let end = start + size;
        while current < end {
            let (desc_tag, desc_size) = read_desc(reader)?;
            match desc_tag {
                0x03 => {
                    es_desc = Some(ESDescriptor::read_desc(reader, desc_size)?);
                }
                _ => break,
            }
            current = reader.stream_position()?;
        }

        if es_desc.is_none() {
            return Err(Error::InvalidData("ESDescriptor not found"));
        }

        skip_bytes_to(reader, start + size)?;

        Ok(EsdsBox {
            version,
            flags,
            es_desc: es_desc.unwrap(),
        })
    }
}

impl<W: Write> WriteBox<&mut W> for EsdsBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        self.es_desc.write_desc(writer)?;

        Ok(size)
    }
}

trait Descriptor: Sized {
    fn desc_tag() -> u8;
    fn desc_size(&self) -> u32;
}

trait ReadDesc<T>: Sized {
    fn read_desc(_: T, size: u32) -> Result<Self>;
}

trait WriteDesc<T>: Sized {
    fn write_desc(&self, _: T) -> Result<u32>;
}

fn read_desc<R: Read>(reader: &mut R) -> Result<(u8, u32)> {
    let tag = reader.read_u8()?;

    let mut size: u32 = 0;
    for _ in 0..4 {
        let b = reader.read_u8()?;
        size = (size << 7) | (b & 0x7F) as u32;
        if b & 0x80 == 0 {
            break;
        }
    }

    Ok((tag, size))
}

fn size_of_length(size: u32) -> u32 {
    match size {
        0x0..=0x7F => 1,
        0x80..=0x3FFF => 2,
        0x4000..=0x1FFFFF => 3,
        _ => 4,
    }
}

fn write_desc<W: Write>(writer: &mut W, tag: u8, size: u32) -> Result<u64> {
    writer.write_u8(tag)?;

    if size as u64 > std::u32::MAX as u64 {
        return Err(Error::InvalidData("invalid descriptor length range"));
    }

    let nbytes = size_of_length(size);

    for i in 0..nbytes {
        let mut b = (size >> ((nbytes - i - 1) * 7)) as u8 & 0x7F;
        if i < nbytes - 1 {
            b |= 0x80;
        }
        writer.write_u8(b)?;
    }

    Ok(1 + nbytes as u64)
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct ESDescriptor {
    pub es_id: u16,

    pub dec_config: DecoderConfigDescriptor,
    pub sl_config: SLConfigDescriptor,
}

impl ESDescriptor {
    pub fn new(config: &AacConfig) -> Self {
        Self {
            es_id: 1,
            dec_config: DecoderConfigDescriptor::new(config),
            sl_config: SLConfigDescriptor::new(),
        }
    }
}

impl Descriptor for ESDescriptor {
    fn desc_tag() -> u8 {
        0x03
    }

    fn desc_size(&self) -> u32 {
        let dec_config_size = self.dec_config.desc_size();
        let sl_config_size = self.sl_config.desc_size();
        3 + 1
            + size_of_length(dec_config_size)
            + dec_config_size
            + 1
            + size_of_length(sl_config_size)
            + sl_config_size
    }
}

impl<R: Read + Seek> ReadDesc<&mut R> for ESDescriptor {
    fn read_desc(reader: &mut R, size: u32) -> Result<Self> {
        let start = reader.stream_position()?;

        let es_id = reader.read_u16::<BigEndian>()?;
        reader.read_u8()?; // XXX flags must be 0

        let mut dec_config = None;
        let mut sl_config = None;

        let mut current = reader.stream_position()?;
        let end = start + size as u64;
        while current < end {
            let (desc_tag, desc_size) = read_desc(reader)?;
            match desc_tag {
                0x04 => {
                    dec_config = Some(DecoderConfigDescriptor::read_desc(reader, desc_size)?);
                }
                0x06 => {
                    sl_config = Some(SLConfigDescriptor::read_desc(reader, desc_size)?);
                }
                _ => {
                    skip_bytes(reader, desc_size as u64)?;
                }
            }
            current = reader.stream_position()?;
        }

        Ok(ESDescriptor {
            es_id,
            dec_config: dec_config.unwrap_or_default(),
            sl_config: sl_config.unwrap_or_default(),
        })
    }
}

impl<W: Write> WriteDesc<&mut W> for ESDescriptor {
    fn write_desc(&self, writer: &mut W) -> Result<u32> {
        let size = self.desc_size();
        write_desc(writer, Self::desc_tag(), size)?;

        writer.write_u16::<BigEndian>(self.es_id)?;
        writer.write_u8(0)?;

        self.dec_config.write_desc(writer)?;
        self.sl_config.write_desc(writer)?;

        Ok(size)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct DecoderConfigDescriptor {
    pub object_type_indication: u8,
    pub stream_type: u8,
    pub up_stream: u8,
    pub buffer_size_db: u32,
    pub max_bitrate: u32,
    pub avg_bitrate: u32,

    pub dec_specific: DecoderSpecificDescriptor,
}

impl DecoderConfigDescriptor {
    pub fn new(config: &AacConfig) -> Self {
        Self {
            object_type_indication: 0x40, // XXX AAC
            stream_type: 0x05,            // XXX Audio
            up_stream: 0,
            buffer_size_db: 0,
            max_bitrate: config.bitrate, // XXX
            avg_bitrate: config.bitrate,
            dec_specific: DecoderSpecificDescriptor::new(config),
        }
    }
}

impl Descriptor for DecoderConfigDescriptor {
    fn desc_tag() -> u8 {
        0x04
    }

    fn desc_size(&self) -> u32 {
        let dec_specific_size = self.dec_specific.desc_size();
        13 + 1 + size_of_length(dec_specific_size) + dec_specific_size
    }
}

impl<R: Read + Seek> ReadDesc<&mut R> for DecoderConfigDescriptor {
    fn read_desc(reader: &mut R, size: u32) -> Result<Self> {
        let start = reader.stream_position()?;

        let object_type_indication = reader.read_u8()?;
        let byte_a = reader.read_u8()?;
        let stream_type = (byte_a & 0xFC) >> 2;
        let up_stream = byte_a & 0x02;
        let buffer_size_db = reader.read_u24::<BigEndian>()?;
        let max_bitrate = reader.read_u32::<BigEndian>()?;
        let avg_bitrate = reader.read_u32::<BigEndian>()?;

        let mut dec_specific = None;

        let mut current = reader.stream_position()?;
        let end = start + size as u64;
        while current < end {
            let (desc_tag, desc_size) = read_desc(reader)?;
            match desc_tag {
                0x05 => {
                    dec_specific = Some(DecoderSpecificDescriptor::read_desc(reader, desc_size)?);
                }
                _ => {
                    skip_bytes(reader, desc_size as u64)?;
                }
            }
            current = reader.stream_position()?;
        }

        Ok(DecoderConfigDescriptor {
            object_type_indication,
            stream_type,
            up_stream,
            buffer_size_db,
            max_bitrate,
            avg_bitrate,
            dec_specific: dec_specific.unwrap_or_default(),
        })
    }
}

impl<W: Write> WriteDesc<&mut W> for DecoderConfigDescriptor {
    fn write_desc(&self, writer: &mut W) -> Result<u32> {
        let size = self.desc_size();
        write_desc(writer, Self::desc_tag(), size)?;

        writer.write_u8(self.object_type_indication)?;
        writer.write_u8((self.stream_type << 2) + (self.up_stream & 0x02) + 1)?; // 1 reserved
        writer.write_u24::<BigEndian>(self.buffer_size_db)?;
        writer.write_u32::<BigEndian>(self.max_bitrate)?;
        writer.write_u32::<BigEndian>(self.avg_bitrate)?;

        self.dec_specific.write_desc(writer)?;

        Ok(size)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct DecoderSpecificDescriptor {
    pub profile: u8,
    pub freq_index: u8,
    pub chan_conf: u8,
}

impl DecoderSpecificDescriptor {
    pub fn new(config: &AacConfig) -> Self {
        Self {
            profile: config.profile as u8,
            freq_index: config.freq_index as u8,
            chan_conf: config.chan_conf as u8,
        }
    }
}

impl Descriptor for DecoderSpecificDescriptor {
    fn desc_tag() -> u8 {
        0x05
    }

    fn desc_size(&self) -> u32 {
        if self.profile < 31 {
            2
        } else {
            3
        }
    }
}

fn get_audio_object_type(byte_a: u8, byte_b: u8) -> u8 {
    let mut profile = byte_a >> 3;
    if profile == 31 {
        profile = 32 + ((byte_a & 7) << 3 | (byte_b >> 5));
    }

    profile
}

fn get_chan_conf<R: Read + Seek>(
    reader: &mut R,
    byte_b: u8,
    freq_index: u8,
    extended_profile: bool,
) -> Result<u8> {
    let chan_conf;
    if freq_index == 15 {
        // Skip the 24 bit sample rate
        let sample_rate = reader.read_u24::<BigEndian>()?;
        chan_conf = ((sample_rate >> 4) & 0x0F) as u8;
    } else if extended_profile {
        let byte_c = reader.read_u8()?;
        chan_conf = (byte_b & 1) << 3 | (byte_c >> 5);
    } else {
        chan_conf = (byte_b >> 3) & 0x0F;
    }

    Ok(chan_conf)
}

impl<R: Read + Seek> ReadDesc<&mut R> for DecoderSpecificDescriptor {
    fn read_desc(reader: &mut R, _size: u32) -> Result<Self> {
        let byte_a = reader.read_u8()?;
        let byte_b = reader.read_u8()?;
        let profile = get_audio_object_type(byte_a, byte_b);
        let freq_index;
        let chan_conf;
        if profile > 31 {
            freq_index = (byte_b >> 1) & 0x0F;
            chan_conf = get_chan_conf(reader, byte_b, freq_index, true)?;
        } else {
            freq_index = ((byte_a & 0x07) << 1) + (byte_b >> 7);
            chan_conf = get_chan_conf(reader, byte_b, freq_index, false)?;
        }

        Ok(DecoderSpecificDescriptor {
            profile,
            freq_index,
            chan_conf,
        })
    }
}

impl<W: Write> WriteDesc<&mut W> for DecoderSpecificDescriptor {
    fn write_desc(&self, writer: &mut W) -> Result<u32> {
        let size = self.desc_size();
        write_desc(writer, Self::desc_tag(), size)?;

        if self.profile < 31 {
            writer.write_u8((self.profile << 3) | (self.freq_index >> 1))?;
            writer.write_u8((self.freq_index << 7) | (self.chan_conf << 3))?;
        } else {
            let profile_minus_32 = self.profile - 32;
            writer.write_u8(31u8 << 3 | profile_minus_32 >> 3)?;
            writer.write_u8(profile_minus_32 << 5 | self.freq_index << 1 | self.chan_conf >> 7)?;
            writer.write_u8((self.chan_conf & 7) << 5)?;
        }

        Ok(size)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct SLConfigDescriptor {}

impl SLConfigDescriptor {
    pub fn new() -> Self {
        SLConfigDescriptor {}
    }
}

impl Descriptor for SLConfigDescriptor {
    fn desc_tag() -> u8 {
        0x06
    }

    fn desc_size(&self) -> u32 {
        1
    }
}

impl<R: Read + Seek> ReadDesc<&mut R> for SLConfigDescriptor {
    fn read_desc(reader: &mut R, _size: u32) -> Result<Self> {
        reader.read_u8()?; // pre-defined

        Ok(SLConfigDescriptor {})
    }
}

impl<W: Write> WriteDesc<&mut W> for SLConfigDescriptor {
    fn write_desc(&self, writer: &mut W) -> Result<u32> {
        let size = self.desc_size();
        write_desc(writer, Self::desc_tag(), size)?;

        writer.write_u8(2)?; // pre-defined
        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mp4box::BoxHeader;
    use std::io::Cursor;

    #[test]
    fn test_mp4a() {
        let src_box = Mp4aBox {
            data_reference_index: 1,
            channelcount: 2,
            samplesize: 16,
            samplerate: FixedPointU16::new(48000),
            esds: Some(EsdsBox {
                version: 0,
                flags: 0,
                es_desc: ESDescriptor {
                    es_id: 2,
                    dec_config: DecoderConfigDescriptor {
                        object_type_indication: 0x40,
                        stream_type: 0x05,
                        up_stream: 0,
                        buffer_size_db: 0,
                        max_bitrate: 67695,
                        avg_bitrate: 67695,
                        dec_specific: DecoderSpecificDescriptor {
                            profile: 2,
                            freq_index: 3,
                            chan_conf: 1,
                        },
                    },
                    sl_config: SLConfigDescriptor::default(),
                },
            }),
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::Mp4aBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = Mp4aBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }

    #[test]
    fn test_mp4a_no_esds() {
        let src_box = Mp4aBox {
            data_reference_index: 1,
            channelcount: 2,
            samplesize: 16,
            samplerate: FixedPointU16::new(48000),
            esds: None,
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::Mp4aBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = Mp4aBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }

    #[test]
    fn test_decoder_specific_descriptor() {
        let test_dec_spec = |src_dec_spec: DecoderSpecificDescriptor| {
            let mut buf = Vec::new();
            let written = src_dec_spec.write_desc(&mut buf).unwrap();
            // expect two extra bytes for the tag and size fields
            assert_eq!(buf.len(), written as usize + 2);

            let mut reader = Cursor::new(&buf);
            let (tag, size) = read_desc(&mut reader).unwrap();
            assert_eq!(5, tag);
            assert_eq!(size, written);

            let dst_dec_spec = DecoderSpecificDescriptor::read_desc(&mut reader, written).unwrap();
            assert_eq!(src_dec_spec, dst_dec_spec);
        };

        test_dec_spec(DecoderSpecificDescriptor {
            profile: 2,    // LC
            freq_index: 4, // 44100
            chan_conf: 2,  // Stereo
        });
        test_dec_spec(DecoderSpecificDescriptor {
            profile: 5,    // SpectralBandReplication (HEv1)
            freq_index: 3, // 48000
            chan_conf: 1,  // Mono
        });
        test_dec_spec(DecoderSpecificDescriptor {
            profile: 29,   // ParametricStereo (HEv2)
            freq_index: 2, // 64000
            chan_conf: 7,  // SevenOne
        });
        test_dec_spec(DecoderSpecificDescriptor {
            profile: 34,   // MpegLayer3
            freq_index: 4, // 44100
            chan_conf: 2,  // Stereo
        });
        test_dec_spec(DecoderSpecificDescriptor {
            profile: 42,    // UnifiedSpeechAudioCoding (xHE)
            freq_index: 11, // 8000
            chan_conf: 6,   // FiveOne
        });
    }
}
