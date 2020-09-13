use std::io::{Read, Seek, SeekFrom, Write};
use serde::{Serialize};

use crate::mp4box::*;
use crate::mp4box::{tfhd::TfhdBox, trun::TrunBox};

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct TrafBox {
    pub tfhd: TfhdBox,
    pub trun: Option<TrunBox>,
}

impl TrafBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::TrafBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        size += self.tfhd.box_size();
        if let Some(ref trun) = self.trun {
            size += trun.box_size();
        }
        size
    }
}

impl Mp4Box for TrafBox {
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

impl<R: Read + Seek> ReadBox<&mut R> for TrafBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut tfhd = None;
        let mut trun = None;

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader { name, size: s } = header;

            match name {
                BoxType::TfhdBox => {
                    tfhd = Some(TfhdBox::read_box(reader, s)?);
                }
                BoxType::TrunBox => {
                    trun = Some(TrunBox::read_box(reader, s)?);
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }

            current = reader.seek(SeekFrom::Current(0))?;
        }

        if tfhd.is_none() {
            return Err(Error::BoxNotFound(BoxType::TfhdBox));
        }

        skip_bytes_to(reader, start + size)?;

        Ok(TrafBox {
            tfhd: tfhd.unwrap(),
            trun,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for TrafBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        self.tfhd.write_box(writer)?;

        Ok(size)
    }
}
