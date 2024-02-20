use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

const RESERVED_DATA_SIZE: u64 = 78;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct EncvBox {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avc1: Option<Avc1Box>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hev1: Option<Hev1Box>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub vp09: Option<Vp09Box>,

    pub sinf: SinfBox,
}

impl EncvBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::EncvBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = 0;
        if let Some(ref avc1) = self.avc1 {
            // HEADER_SIZE intentionally omitted
            size += avc1.box_size();
        } else if let Some(ref hev1) = self.hev1 {
            // HEADER_SIZE intentionally omitted
            size += hev1.box_size();
        } else if let Some(ref vp09) = self.vp09 {
            // HEADER_SIZE intentionally omitted
            size += vp09.box_size();
        } else {
            size += HEADER_SIZE + RESERVED_DATA_SIZE;
        }
        size += self.sinf.box_size();
        size
    }
}

impl Mp4Box for EncvBox {
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
        let child_summary = if let Some(ref avc1) = self.avc1 {
            avc1.summary()
        } else if let Some(ref hev1) = self.hev1 {
            hev1.summary()
        } else if let Some(ref vp09) = self.vp09 {
            vp09.summary()
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

impl<R: Read + Seek> ReadBox<&mut R> for EncvBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut avc1 = None;
        let mut hev1 = None;
        let mut vp09 = None;
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
                    "encv box contains a box with a larger size than it",
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
        match original_format {
            BoxType::Avc1Box => {
                avc1 = Some(Avc1Box::read_box(reader, size)?);
            }
            BoxType::Hev1Box => {
                hev1 = Some(Hev1Box::read_box(reader, size)?);
            }
            BoxType::Vp09Box => {
                vp09 = Some(Vp09Box::read_box(reader, size)?);
            }
            _ => (),
        }

        skip_bytes_to(reader, start + size)?;

        Ok(EncvBox {
            avc1,
            hev1,
            vp09,
            sinf,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for EncvBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        if let Some(ref avc1) = self.avc1 {
            // the encv box header is used, so the header from this box
            // must be removed
            let mut buf = Vec::with_capacity(avc1.box_size() as usize);
            avc1.write_box(&mut buf)?;
            writer.write_all(&buf[HEADER_SIZE as usize..])?;
        } else if let Some(ref hev1) = self.hev1 {
            // the encv box header is used, so the header from this box
            // must be removed
            let mut buf = Vec::with_capacity(hev1.box_size() as usize);
            hev1.write_box(&mut buf)?;
            writer.write_all(&buf[HEADER_SIZE as usize..])?;
        } else if let Some(ref vp09) = self.vp09 {
            // the encv box header is used, so the header from this box
            // must be removed
            let mut buf = Vec::with_capacity(vp09.box_size() as usize);
            vp09.write_box(&mut buf)?;
            writer.write_all(&buf[HEADER_SIZE as usize..])?;
        } else {
            writer.write_all(&[0; RESERVED_DATA_SIZE as usize])?;
        }

        self.sinf.write_box(writer)?;

        Ok(size)
    }
}
