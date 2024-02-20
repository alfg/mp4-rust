use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct SchiBox {
    pub tenc: Option<TencBox>,
}

impl SchiBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::SchiBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        if let Some(ref tenc) = self.tenc {
            size += tenc.get_size();
        }
        size
    }
}

impl Mp4Box for SchiBox {
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

impl<R: Read + Seek> ReadBox<&mut R> for SchiBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut tenc = None;

        let mut current = reader.stream_position()?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader { name, size: s } = header;
            if s > size {
                return Err(Error::InvalidData(
                    "schi box contains a box with a larger size than it",
                ));
            }

            match name {
                BoxType::TencBox => {
                    tenc = Some(TencBox::read_box(reader, s)?);
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }

            current = reader.stream_position()?;
        }

        skip_bytes_to(reader, start + size)?;

        Ok(SchiBox { tenc })
    }
}

impl<W: Write> WriteBox<&mut W> for SchiBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        if let Some(ref tenc) = self.tenc {
            tenc.write_box(writer)?;
        }

        Ok(size)
    }
}
