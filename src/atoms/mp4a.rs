use std::io::{Seek, Read, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::*;
use crate::atoms::*;


#[derive(Debug, PartialEq)]
pub struct Mp4aBox {
    pub data_reference_index: u16,
    pub channel_count: u16,
    pub samplesize: u16,
    pub samplerate: u32,
    pub esds: EsdsBox,
}

impl Default for Mp4aBox {
    fn default() -> Self {
        Mp4aBox {
            data_reference_index: 0,
            channel_count: 2,
            samplesize: 16,
            samplerate: 0, // XXX
            esds: EsdsBox::default(),
        }
    }
}

impl Mp4Box for Mp4aBox {
    fn box_type() -> BoxType {
        BoxType::Mp4aBox
    }

    fn box_size(&self) -> u64 {
        HEADER_SIZE + 8 + 74 + self.esds.box_size()
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for Mp4aBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = get_box_start(reader)?;

        reader.read_u32::<BigEndian>()?; // reserved
        reader.read_u16::<BigEndian>()?; // reserved
        let data_reference_index = reader.read_u16::<BigEndian>()?;

        reader.read_u64::<BigEndian>()?; // reserved
        let channel_count = reader.read_u16::<BigEndian>()?;
        let samplesize = reader.read_u16::<BigEndian>()?;
        reader.read_u32::<BigEndian>()?; // pre-defined, reserved
        let samplerate = reader.read_u32::<BigEndian>()?;

        let header = BoxHeader::read(reader)?;
        let BoxHeader{ name, size: s } = header;
        if name == BoxType::EsdsBox {
            let esds = EsdsBox::read_box(reader, s)?;

            skip_read_to(reader, start + size)?;

            Ok(Mp4aBox {
                data_reference_index,
                channel_count,
                samplesize,
                samplerate,
                esds,
            })
        } else {
            Err(Error::InvalidData("esds not found"))
        }
    }
}

impl<W: Write> WriteBox<&mut W> for Mp4aBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(Self::box_type(), size).write(writer)?;

        writer.write_u32::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(self.data_reference_index)?;

        writer.write_u64::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(self.channel_count)?;
        writer.write_u16::<BigEndian>(self.samplesize)?;
        writer.write_u32::<BigEndian>(0)?; // reserved
        writer.write_u32::<BigEndian>(self.samplerate)?;

        self.esds.write_box(writer)?;

        Ok(size)
    }
}


#[derive(Debug, Default, PartialEq)]
pub struct EsdsBox {
    pub version: u8,
    pub flags: u32,
    pub es_desc: ESDescriptor,
}

impl Mp4Box for EsdsBox {
    fn box_type() -> BoxType {
        BoxType::EsdsBox
    }

    fn box_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for EsdsBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = get_box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        let es_desc = ESDescriptor::read_desc(reader)?;

        skip_read_to(reader, start + size)?;

        Ok(EsdsBox {
            version,
            flags,
            es_desc,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for EsdsBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(Self::box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        Ok(size)
    }
}


trait Descriptor: Sized {
    fn desc_tag() -> u8;
    fn desc_size() -> u32;
}

trait ReadDesc<T>: Sized {
    fn read_desc(_: T) -> Result<Self>;
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

fn write_desc<W: Write>(writer: &mut W, tag: u8, size: u32) -> Result<u64> {
    writer.write_u8(tag)?;

    if size > 0x0FFFFFFF {
        return Err(Error::InvalidData("invalid descriptor length range"));
    }

    let nbytes = match size {
        0x0..=0x7F => 1,
        0x80..=0x3FFF => 2,
        0x4000..=0x1FFFFF => 3,
        _ => 4,
    };

    for i in 0..nbytes {
        let mut b = (size >> ((3 - i) * 7)) as u8 & 0x7F;
        if i < nbytes - 1 {
            b |= 0x80;
        }
        writer.write_u8(b)?;
    }

    Ok(1 + nbytes)
}


#[derive(Debug, Default, PartialEq)]
pub struct ESDescriptor {
    pub tag: u8,
    pub size: u32,

    pub es_id: u16,

    pub dec_config: DecoderConfigDescriptor,
    pub sl_config: SLConfigDescriptor,
}

impl Descriptor for ESDescriptor {
    fn desc_tag() -> u8 {
        0x03
    }

    // XXX size > 0x7F
    fn desc_size() -> u32 {
        2 + 3
            + DecoderConfigDescriptor::desc_size()
            + SLConfigDescriptor::desc_size()
    }
}

impl<R: Read + Seek> ReadDesc<&mut R> for ESDescriptor {
    fn read_desc(reader: &mut R) -> Result<Self> {
        let (tag, size) = read_desc(reader)?;
        if tag != Self::desc_tag() {
            return Err(Error::InvalidData("ESDescriptor not found"));
        }

        let es_id = reader.read_u16::<BigEndian>()?;
        reader.read_u8()?; // XXX flags must be 0

        let dec_config = DecoderConfigDescriptor::read_desc(reader)?;
        let sl_config = SLConfigDescriptor::read_desc(reader)?;

        Ok(ESDescriptor {
            tag,
            size,
            es_id,
            dec_config,
            sl_config,
        })
    }
}

impl<W: Write> WriteDesc<&mut W> for ESDescriptor {
    fn write_desc(&self, writer: &mut W) -> Result<u32> {
        write_desc(writer, self.tag, self.size)?;

        Ok(self.size)
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct DecoderConfigDescriptor {
    pub tag: u8,
    pub size: u32,

    pub object_type_indication: u8,
    pub stream_type: u8,
    pub up_stream: u8,
    pub buffer_size_db: u32,
    pub max_bitrate: u32,
    pub avg_bitrate: u32,

    pub dec_specific: DecoderSpecificDescriptor,
}

impl Descriptor for DecoderConfigDescriptor {
    fn desc_tag() -> u8 {
        0x04
    }

    // XXX size > 0x7F
    fn desc_size() -> u32 {
        2 + 13 + DecoderSpecificDescriptor::desc_size()
    }
}

impl<R: Read + Seek> ReadDesc<&mut R> for DecoderConfigDescriptor {
    fn read_desc(reader: &mut R) -> Result<Self> {
        let (tag, size) = read_desc(reader)?;
        if tag != Self::desc_tag() {
            return Err(Error::InvalidData("DecoderConfigDescriptor not found"));
        }

        let object_type_indication = reader.read_u8()?;
        let byte_a = reader.read_u8()?;
        let stream_type = byte_a & 0xFC;
        let up_stream = byte_a & 0x02;
        let buffer_size_db = reader.read_u24::<BigEndian>()?;
        let max_bitrate = reader.read_u32::<BigEndian>()?;
        let avg_bitrate = reader.read_u32::<BigEndian>()?;

        let dec_specific = DecoderSpecificDescriptor::read_desc(reader)?;

        // XXX skip_read
        for _ in DecoderConfigDescriptor::desc_size()..size-1 {
            reader.read_u8()?;
        }

        Ok(DecoderConfigDescriptor {
            tag,
            size,
            object_type_indication,
            stream_type,
            up_stream,
            buffer_size_db,
            max_bitrate,
            avg_bitrate,
            dec_specific,
        })
    }
}

impl<W: Write> WriteDesc<&mut W> for DecoderConfigDescriptor {
    fn write_desc(&self, writer: &mut W) -> Result<u32> {
        write_desc(writer, self.tag, self.size)?;

        Ok(self.size)
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct DecoderSpecificDescriptor {
    pub tag: u8,
    pub size: u32,
    pub profile: u8,
    pub freq_index: u8,
    pub chan_conf: u8,
}

impl Descriptor for DecoderSpecificDescriptor {
    fn desc_tag() -> u8 {
        0x05
    }

    // XXX size > 0x7F
    fn desc_size() -> u32 {
        2 + 2
    }
}

impl<R: Read + Seek> ReadDesc<&mut R> for DecoderSpecificDescriptor {
    fn read_desc(reader: &mut R) -> Result<Self> {
        let (tag, size) = read_desc(reader)?;
        if tag != Self::desc_tag() {
            return Err(Error::InvalidData("DecoderSpecificDescriptor not found"));
        }

        let byte_a = reader.read_u8()?;
        let byte_b = reader.read_u8()?;
        let profile = byte_a >> 3;
        let freq_index = ((byte_a & 0x07) << 1) + (byte_b >> 7);
        let chan_conf = (byte_b >> 3) & 0x0F;

        Ok(DecoderSpecificDescriptor {
            tag,
            size,
            profile,
            freq_index,
            chan_conf,
        })
    }
}

impl<W: Write> WriteDesc<&mut W> for DecoderSpecificDescriptor {
    fn write_desc(&self, writer: &mut W) -> Result<u32> {
        write_desc(writer, self.tag, self.size)?;

        Ok(self.size)
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct SLConfigDescriptor {
    pub tag: u8,
    pub size: u32,
}

impl Descriptor for SLConfigDescriptor {
    fn desc_tag() -> u8 {
        0x06
    }

    // XXX size > 0x7F
    fn desc_size() -> u32 {
        2 + 1
    }
}

impl<R: Read + Seek> ReadDesc<&mut R> for SLConfigDescriptor {
    fn read_desc(reader: &mut R) -> Result<Self> {
        let (tag, size) = read_desc(reader)?;
        if tag != Self::desc_tag() {
            return Err(Error::InvalidData("SLConfigDescriptor not found"));
        }

        reader.read_u8()?; // pre-defined

        Ok(SLConfigDescriptor {
            tag,
            size,
        })
    }
}

impl<W: Write> WriteDesc<&mut W> for SLConfigDescriptor {
    fn write_desc(&self, writer: &mut W) -> Result<u32> {
        write_desc(writer, self.tag, self.size)?;

        Ok(self.size)
    }
}
