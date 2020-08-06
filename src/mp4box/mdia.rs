use std::io::{Read, Seek, SeekFrom, Write};

use crate::mp4box::*;
use crate::mp4box::{hdlr::HdlrBox, mdhd::MdhdBox, minf::MinfBox};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MdiaBox {
    pub mdhd: MdhdBox,
    pub hdlr: HdlrBox,
    pub minf: MinfBox,
}

impl Mp4Box for MdiaBox {
    fn box_type() -> BoxType {
        BoxType::MdiaBox
    }

    fn box_size(&self) -> u64 {
        HEADER_SIZE + self.mdhd.box_size() + self.hdlr.box_size() + self.minf.box_size()
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for MdiaBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut mdhd = None;
        let mut hdlr = None;
        let mut minf = None;

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader { name, size: s } = header;

            match name {
                BoxType::MdhdBox => {
                    mdhd = Some(MdhdBox::read_box(reader, s)?);
                }
                BoxType::HdlrBox => {
                    hdlr = Some(HdlrBox::read_box(reader, s)?);
                }
                BoxType::MinfBox => {
                    minf = Some(MinfBox::read_box(reader, s)?);
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }

            current = reader.seek(SeekFrom::Current(0))?;
        }

        if mdhd.is_none() {
            return Err(Error::BoxNotFound(BoxType::MdhdBox));
        }
        if hdlr.is_none() {
            return Err(Error::BoxNotFound(BoxType::HdlrBox));
        }
        if minf.is_none() {
            return Err(Error::BoxNotFound(BoxType::MinfBox));
        }

        skip_bytes_to(reader, start + size)?;

        Ok(MdiaBox {
            mdhd: mdhd.unwrap(),
            hdlr: hdlr.unwrap(),
            minf: minf.unwrap(),
        })
    }
}

impl<W: Write> WriteBox<&mut W> for MdiaBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(Self::box_type(), size).write(writer)?;

        self.mdhd.write_box(writer)?;
        self.hdlr.write_box(writer)?;
        self.minf.write_box(writer)?;

        Ok(size)
    }
}
