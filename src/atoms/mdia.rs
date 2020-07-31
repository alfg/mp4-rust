use std::io::{Seek, SeekFrom, Read, Write};

use crate::*;
use crate::atoms::*;
use crate::atoms::{mdhd::MdhdBox, hdlr::HdlrBox, minf::MinfBox};


#[derive(Debug, Default)]
pub struct MdiaBox {
    pub mdhd: Option<MdhdBox>,
    pub hdlr: Option<HdlrBox>,
    pub minf: Option<MinfBox>,
}

impl MdiaBox {
    pub(crate) fn new() -> MdiaBox {
        Default::default()
    }
}

impl Mp4Box for MdiaBox {
    fn box_type() -> BoxType {
        BoxType::MdiaBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        if let Some(ref mdhd) = self.mdhd {
            size += mdhd.box_size();
        }
        if let Some(ref hdlr) = self.hdlr {
            size += hdlr.box_size();
        }
        if let Some(ref minf) = self.minf {
            size += minf.box_size();
        }
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for MdiaBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = get_box_start(reader)?;

        let mut mdia = MdiaBox::new();

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader{ name, size: s } = header;

            match name {
                BoxType::MdhdBox => {
                    let mdhd = MdhdBox::read_box(reader, s)?;
                    mdia.mdhd = Some(mdhd);
                }
                BoxType::HdlrBox => {
                    let hdlr = HdlrBox::read_box(reader, s)?;
                    mdia.hdlr = Some(hdlr);
                }
                BoxType::MinfBox => {
                    let minf = MinfBox::read_box(reader, s)?;
                    mdia.minf = Some(minf);
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }

            current = reader.seek(SeekFrom::Current(0))?;
        }

        skip_read_to(reader, start + size)?;

        Ok(mdia)
    }
}

impl<W: Write> WriteBox<&mut W> for MdiaBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(Self::box_type(), size).write(writer)?;

        if let Some(ref mdhd) = self.mdhd {
            mdhd.write_box(writer)?;
        }
        if let Some(ref hdlr) = self.hdlr {
            hdlr.write_box(writer)?;
        }
        if let Some(ref minf) = self.minf {
            minf.write_box(writer)?;
        }

        Ok(size)
    }
}
