use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
#[cfg(feature = "use_serde")]
use serde::Serialize;
use std::convert::TryInto;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

// Track Fragment Decode Time box
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "use_serde", derive(Serialize))]
pub struct TfdtBox {
    pub version: u8,

    pub base_media_decode_time: u64,
}

impl Default for TfdtBox {
    fn default() -> Self {
        Self {
            version: 0,
            base_media_decode_time: 0,
        }
    }
}

impl TfdtBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::TfdtBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE;
        match self.version {
            0 => size += 4,
            1 => size += 8,
            _ => panic!(),
        }
        size
    }
}

impl Mp4Box for TfdtBox {
    fn box_type(&self) -> BoxType {
        self.get_type()
    }

    fn box_size(&self) -> u64 {
        self.get_size()
    }

    #[cfg(feature = "use_serde")]
    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("decode_time={}", self.base_media_decode_time);
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for TfdtBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let _start = box_start(reader)?;
        let (version, _flags) = read_box_header_ext(reader)?;

        let base_media_decode_time = match version {
            0 => reader.read_u32::<BigEndian>()? as u64,
            1 => reader.read_u64::<BigEndian>()?,
            _ => panic!(),
        };

        Ok(TfdtBox {
            version,
            base_media_decode_time,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for TfdtBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, 0)?;

        match self.version {
            0 => writer.write_u32::<BigEndian>(self.base_media_decode_time.try_into().unwrap())?,
            1 => writer.write_u64::<BigEndian>(self.base_media_decode_time)?,
            _ => panic!(),
        }

        Ok(size)
    }
}
