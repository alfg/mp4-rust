use std::io::{Read, Seek, SeekFrom, Write};
use serde::{Serialize};

use crate::mp4box::*;
use crate::mp4box::{mehd::MehdBox, trex::TrexBox};

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct MvexBox {
    pub mehd: MehdBox,
    pub trex: TrexBox,
}

impl MvexBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::MdiaBox
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE + self.mehd.box_size() + self.trex.box_size()
    }
}

impl Mp4Box for MvexBox {
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
        let s = format!("");
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

        if mehd.is_none() {
            return Err(Error::BoxNotFound(BoxType::MehdBox));
        }
        if trex.is_none() {
            return Err(Error::BoxNotFound(BoxType::TrexBox));
        }

        skip_bytes_to(reader, start + size)?;

        Ok(MvexBox {
            mehd: mehd.unwrap(),
            trex: trex.unwrap(),
        })
    }
}

impl<W: Write> WriteBox<&mut W> for MvexBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        self.mehd.write_box(writer)?;
        self.trex.write_box(writer)?;

        Ok(size)
    }
}
