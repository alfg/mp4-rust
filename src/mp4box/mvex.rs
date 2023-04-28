use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;
use crate::mp4box::{mehd::MehdBox, trex::TrexBox};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct MvexBox {
    pub mehd: Option<MehdBox>,

    #[serde(rename = "trex")]
    pub trexs: Vec<TrexBox>,
}

impl MvexBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::MdiaBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE;

        size += self.mehd.as_ref().map_or(0,|x| x.box_size());

        for trex in self.trexs.iter() {
            size += trex.box_size();
        }

        size
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
        let mut trexs = Vec::new();

        let mut current = reader.stream_position()?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader { name, size: s } = header;
            if s > size {
                return Err(Error::InvalidData(
                    "mvex box contains a box with a larger size than it",
                ));
            }

            match name {
                BoxType::MehdBox => {
                    mehd = Some(MehdBox::read_box(reader, s)?);
                }
                BoxType::TrexBox => {
                    trexs.push(TrexBox::read_box(reader, s)?);
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }

            current = reader.stream_position()?;
        }

        if trexs.is_empty() {
            return Err(Error::BoxNotFound(BoxType::TrexBox));
        }

        skip_bytes_to(reader, start + size)?;

        Ok(MvexBox {
            mehd,
            trexs,
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

        for trex in self.trexs.iter() {
            trex.write_box(writer)?;
        }

        Ok(size)
    }
}
