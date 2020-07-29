use std::io::{BufReader, SeekFrom, Seek, Read, BufWriter, Write};
use byteorder::{BigEndian, ReadBytesExt};

use crate::*;
use crate::atoms::{avc::Avc1Box, mp4a::Mp4aBox};


#[derive(Debug)]
pub struct StsdBox {
    pub version: u8,
    pub flags: u32,
    pub avc1: Option<Avc1Box>,
    pub mp4a: Option<Mp4aBox>,
}

impl Mp4Box for StsdBox {
    fn box_type(&self) -> BoxType {
        BoxType::StsdBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE;
        if let Some(avc1) = &self.avc1 {
            size += avc1.box_size();
        } else if let Some(mp4a) = &self.mp4a {
            size += mp4a.box_size();
        }
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for StsdBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0))?; // Current cursor position.

        let (version, flags) = read_box_header_ext(reader)?;

        let _entry_count = reader.read_u32::<BigEndian>()?;

        let mut avc1 = None;
        let mut mp4a = None;

        // Get box header.
        let header = read_box_header(reader, 0)?;
        let BoxHeader{ name, size: s } = header;

        match name {
            BoxType::Avc1Box => {
                avc1 = Some(Avc1Box::read_box(reader, s)?);
            }
            BoxType::Mp4aBox => {
                mp4a = Some(Mp4aBox::read_box(reader, s)?);
            }
            _ => {}
        }

        skip_read(reader, current, size)?;

        Ok(StsdBox {
            version,
            flags,
            avc1,
            mp4a,
        })
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for StsdBox {
    fn write_box(&self, _writer: &mut BufWriter<W>) -> Result<u64> {
        // TODO
        Ok(0)
    }
}
