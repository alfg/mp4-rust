use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::meta::MetaBox;
use crate::mp4box::*;
use crate::mp4box::{edts::EdtsBox, mdia::MdiaBox, tkhd::TkhdBox};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct TrakBox {
    pub tkhd: TkhdBox,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub edts: Option<EdtsBox>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<MetaBox>,

    pub mdia: MdiaBox,
}

impl TrakBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::TrakBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        size += self.tkhd.box_size();
        if let Some(ref edts) = self.edts {
            size += edts.box_size();
        }
        size += self.mdia.box_size();
        size
    }
}

impl Mp4Box for TrakBox {
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

impl<R: Read + Seek> ReadBox<&mut R> for TrakBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut tkhd = None;
        let mut edts = None;
        let mut meta = None;
        let mut mdia = None;

        let mut current = reader.stream_position()?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader { name, size: s } = header;

            match name {
                BoxType::TkhdBox => {
                    tkhd = Some(TkhdBox::read_box(reader, s)?);
                }
                BoxType::EdtsBox => {
                    edts = Some(EdtsBox::read_box(reader, s)?);
                }
                BoxType::MetaBox => {
                    meta = Some(MetaBox::read_box(reader, s)?);
                }
                BoxType::MdiaBox => {
                    mdia = Some(MdiaBox::read_box(reader, s)?);
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }

            current = reader.stream_position()?;
        }

        if tkhd.is_none() {
            return Err(Error::BoxNotFound(BoxType::TkhdBox));
        }
        if mdia.is_none() {
            return Err(Error::BoxNotFound(BoxType::MdiaBox));
        }

        skip_bytes_to(reader, start + size)?;

        Ok(TrakBox {
            tkhd: tkhd.unwrap(),
            edts,
            meta,
            mdia: mdia.unwrap(),
        })
    }
}

impl<W: Write> WriteBox<&mut W> for TrakBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        self.tkhd.write_box(writer)?;
        if let Some(ref edts) = self.edts {
            edts.write_box(writer)?;
        }
        self.mdia.write_box(writer)?;

        Ok(size)
    }
}
