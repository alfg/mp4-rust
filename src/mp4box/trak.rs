use std::io::{Read, Seek, SeekFrom, Write};

use crate::mp4box::*;
use crate::mp4box::{edts::EdtsBox, mdia::MdiaBox, tkhd::TkhdBox};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TrakBox {
    pub tkhd: TkhdBox,
    pub edts: Option<EdtsBox>,
    pub mdia: MdiaBox,
}

impl Mp4Box for TrakBox {
    fn box_type() -> BoxType {
        BoxType::TrakBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        size += self.tkhd.box_size();
        if let Some(ref edts) = self.edts {
            size += edts.box_size();
        }
        size += self.mdia.box_size();
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for TrakBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut tkhd = None;
        let mut edts = None;
        let mut mdia = None;

        let mut current = reader.seek(SeekFrom::Current(0))?;
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
                BoxType::MdiaBox => {
                    mdia = Some(MdiaBox::read_box(reader, s)?);
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }

            current = reader.seek(SeekFrom::Current(0))?;
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
            mdia: mdia.unwrap(),
        })
    }
}

impl<W: Write> WriteBox<&mut W> for TrakBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(Self::box_type(), size).write(writer)?;

        self.tkhd.write_box(writer)?;
        if let Some(ref edts) = self.edts {
            edts.write_box(writer)?;
        }
        self.mdia.write_box(writer)?;

        Ok(size)
    }
}
