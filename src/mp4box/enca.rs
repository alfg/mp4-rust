use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

const RESERVED_DATA_SIZE: u64 = 28;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct EncaBox {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mp4a: Option<Mp4aBox>,

    pub sinf: SinfBox,
}

impl EncaBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::EncaBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = 0;
        if let Some(ref mp4a) = self.mp4a {
            // HEADER_SIZE intentionally omitted
            size += mp4a.box_size();
        } else {
            size += HEADER_SIZE + RESERVED_DATA_SIZE;
        }
        size += self.sinf.box_size();
        size
    }
}

impl Mp4Box for EncaBox {
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
        let child_summary = if let Some(ref mp4a) = self.mp4a {
            mp4a.summary()
        } else {
            Err(Error::InvalidData(""))
        };
        let mut s = format!("original_format={}", &self.sinf.frma.original_format);
        if let Ok(summary) = child_summary {
            s.push(' ');
            s.push_str(&summary);
        }
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for EncaBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut mp4a = None;
        let mut sinf = None;

        // skip current container items
        skip_bytes(reader, RESERVED_DATA_SIZE)?;

        let mut current = reader.stream_position()?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader { name, size: s } = header;
            if s > size {
                return Err(Error::InvalidData(
                    "enca box contains a box with a larger size than it",
                ));
            }

            match name {
                BoxType::SinfBox => {
                    sinf = Some(SinfBox::read_box(reader, s)?);
                    break;
                }
                _ => {
                    skip_box(reader, s)?;
                }
            }

            current = reader.stream_position()?;
        }

        let sinf = sinf.ok_or(Error::BoxNotFound(BoxType::SinfBox))?;

        reader.seek(SeekFrom::Start(start + HEADER_SIZE))?;

        let original_format: BoxType = sinf.frma.original_format.into();
        if original_format == BoxType::Mp4aBox {
            mp4a = Some(Mp4aBox::read_box(reader, size)?);
        }

        skip_bytes_to(reader, start + size)?;

        Ok(EncaBox { mp4a, sinf })
    }
}

impl<W: Write> WriteBox<&mut W> for EncaBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        if let Some(ref mp4a) = self.mp4a {
            // the enca box header is used, so the header from this box
            // must be removed
            let mut buf = Vec::with_capacity(mp4a.box_size() as usize);
            mp4a.write_box(&mut buf)?;
            writer.write_all(&buf[HEADER_SIZE as usize..])?;
        } else {
            writer.write_all(&[0; RESERVED_DATA_SIZE as usize])?;
        }

        self.sinf.write_box(writer)?;

        Ok(size)
    }
}
