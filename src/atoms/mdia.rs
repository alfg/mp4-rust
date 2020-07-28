use std::io::{BufReader, SeekFrom, Seek, Read, BufWriter, Write};

use crate::*;
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
    fn box_type(&self) -> BoxType {
        BoxType::MdiaBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        if let Some(mdhd) = &self.mdhd {
            size += mdhd.box_size();
        }
        if let Some(hdlr) = &self.hdlr {
            size += hdlr.box_size();
        }
        if let Some(minf) = &self.minf {
            size += minf.box_size();
        }
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for MdiaBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0))?; // Current cursor position.
        let mut mdia = MdiaBox::new();

        let start = 0u64;
        while start < size {
            // Get box header.
            let header = read_box_header(reader, start)?;
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
                _ => break
            }
        }
        skip_read(reader, current, size)?;

        Ok(mdia)
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for MdiaBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

        if let Some(mdhd) = &self.mdhd {
            mdhd.write_box(writer)?;
        }
        if let Some(hdlr) = &self.hdlr {
            hdlr.write_box(writer)?;
        }
        if let Some(minf) = &self.minf {
            minf.write_box(writer)?;
        }

        Ok(size)
    }
}
