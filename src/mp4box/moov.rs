use std::io::{Read, Seek, SeekFrom, Write};
use serde::{Serialize};

use crate::mp4box::*;
use crate::mp4box::{mvhd::MvhdBox, trak::TrakBox};

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct MoovBox {
    pub mvhd: MvhdBox,
    pub traks: Vec<TrakBox>,
}

impl MoovBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::MoovBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE + self.mvhd.box_size();
        for trak in self.traks.iter() {
            size += trak.box_size();
        }
        size
    }
}

impl Mp4Box for MoovBox {
    fn box_type(&self) -> BoxType {
        return self.get_type();
    }

    fn box_size(&self) -> u64 {
        return self.get_size();
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("traks={}", self.traks.len());
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for MoovBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut mvhd = None;
        let mut traks = Vec::new();

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader { name, size: s } = header;

            match name {
                BoxType::MvhdBox => {
                    mvhd = Some(MvhdBox::read_box(reader, s)?);
                }
                BoxType::TrakBox => {
                    let trak = TrakBox::read_box(reader, s)?;
                    traks.push(trak);
                }
                BoxType::UdtaBox => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }

            current = reader.seek(SeekFrom::Current(0))?;
        }

        if mvhd.is_none() {
            return Err(Error::BoxNotFound(BoxType::MvhdBox));
        }

        skip_bytes_to(reader, start + size)?;

        Ok(MoovBox {
            mvhd: mvhd.unwrap(),
            traks,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for MoovBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        self.mvhd.write_box(writer)?;
        for trak in self.traks.iter() {
            trak.write_box(writer)?;
        }
        Ok(0)
    }
}
