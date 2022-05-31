use serde::Serialize;
use std::io::{Read, Seek, SeekFrom, Write};

use crate::mp4box::*;
use crate::mp4box::{mehd::MehdBox, trex::TrexBox};

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct MvexBox {
    pub mehd: Option<MehdBox>,
    pub trex: TrexBox,
}

impl MvexBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::MdiaBox
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE + self.mehd.as_ref().map(|x| x.box_size()).unwrap_or(0) + self.trex.box_size()
    }
}

impl Mp4Box for MvexBox {
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
        let s = String::new();
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for MvexBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut mehd = None;
        let mut trex = None;

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader { name, size: s } = header;

            match name {
                BoxType::MehdBox => {
                    mehd = Some(MehdBox::read_box(reader, s)?);
                }
                BoxType::TrexBox => {
                    trex = Some(TrexBox::read_box(reader, s)?);
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }

            current = reader.seek(SeekFrom::Current(0))?;
        }

        if trex.is_none() {
            return Err(Error::BoxNotFound(BoxType::TrexBox));
        }

        skip_bytes_to(reader, start + size)?;

        Ok(MvexBox {
            mehd,
            trex: trex.unwrap(),
        })
    }
}

impl<W: Write> WriteBox<&mut W> for MvexBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        if let Some(mehd) = &self.mehd {
            mehd.write_box(writer)?;
        }
        self.trex.write_box(writer)?;

        Ok(size)
    }
}
