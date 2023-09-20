use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct SinfBox {
    pub frma: FrmaBox,
    pub schm: Option<SchmBox>,
    pub schi: Option<SchiBox>,
}

impl SinfBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::SinfBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE + self.frma.get_size();
        if let Some(ref schm) = self.schm {
            size += schm.get_size();
        }
        if let Some(ref schi) = self.schi {
            size += schi.get_size();
        }
        size
    }
}

impl Mp4Box for SinfBox {
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

impl<R: Read + Seek> ReadBox<&mut R> for SinfBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut frma = None;
        let mut schm = None;
        let mut schi = None;

        let mut current = reader.stream_position()?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader { name, size: s } = header;
            if s > size {
                return Err(Error::InvalidData(
                    "sinf box contains a box with a larger size than it",
                ));
            }

            match name {
                BoxType::FrmaBox => {
                    frma = Some(FrmaBox::read_box(reader, s)?);
                }
                BoxType::SchmBox => {
                    schm = Some(SchmBox::read_box(reader, s)?);
                }
                BoxType::SchiBox => {
                    schi = Some(SchiBox::read_box(reader, s)?);
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }

            current = reader.stream_position()?;
        }

        if frma.is_none() {
            return Err(Error::BoxNotFound(BoxType::FrmaBox));
        }

        skip_bytes_to(reader, start + size)?;

        Ok(SinfBox {
            frma: frma.unwrap(),
            schm,
            schi,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for SinfBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();

        BoxHeader::new(self.box_type(), size).write(writer)?;

        self.frma.write_box(writer)?;

        if let Some(schm) = &self.schm {
            schm.write_box(writer)?;
        }

        if let Some(schi) = &self.schi {
            schi.write_box(writer)?;
        }

        Ok(size)
    }
}
