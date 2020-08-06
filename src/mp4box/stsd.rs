use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, Write};

use crate::mp4box::*;
use crate::mp4box::{avc1::Avc1Box, mp4a::Mp4aBox};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StsdBox {
    pub version: u8,
    pub flags: u32,
    pub avc1: Option<Avc1Box>,
    pub mp4a: Option<Mp4aBox>,
}

impl Mp4Box for StsdBox {
    fn box_type() -> BoxType {
        BoxType::StsdBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE + 4;
        if let Some(ref avc1) = self.avc1 {
            size += avc1.box_size();
        } else if let Some(ref mp4a) = self.mp4a {
            size += mp4a.box_size();
        }
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for StsdBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        reader.read_u32::<BigEndian>()?; // XXX entry_count

        let mut avc1 = None;
        let mut mp4a = None;

        // Get box header.
        let header = BoxHeader::read(reader)?;
        let BoxHeader { name, size: s } = header;

        match name {
            BoxType::Avc1Box => {
                avc1 = Some(Avc1Box::read_box(reader, s)?);
            }
            BoxType::Mp4aBox => {
                mp4a = Some(Mp4aBox::read_box(reader, s)?);
            }
            _ => {}
        }

        skip_bytes_to(reader, start + size)?;

        Ok(StsdBox {
            version,
            flags,
            avc1,
            mp4a,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for StsdBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(Self::box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        writer.write_u32::<BigEndian>(1)?; // entry_count

        if let Some(ref avc1) = self.avc1 {
            avc1.write_box(writer)?;
        } else if let Some(ref mp4a) = self.mp4a {
            mp4a.write_box(writer)?;
        }

        Ok(size)
    }
}
