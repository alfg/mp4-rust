use std::io::{BufReader, SeekFrom, Seek, Read, BufWriter, Write};

use crate::*;
use crate::atoms::{tkhd::TkhdBox, edts::EdtsBox, mdia::MdiaBox};


#[derive(Debug, Default)]
pub struct TrakBox {
    pub tkhd: Option<TkhdBox>,
    pub edts: Option<EdtsBox>,
    pub mdia: Option<MdiaBox>,
}

impl TrakBox {
    pub(crate) fn new() -> TrakBox {
        Default::default()
    }
}

impl Mp4Box for TrakBox {
    fn box_type(&self) -> BoxType {
        BoxType::TrakBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        if let Some(tkhd) = &self.tkhd {
            size += tkhd.box_size();
        }
        if let Some(edts) = &self.edts {
            size += edts.box_size();
        }
        if let Some(mdia) = &self.mdia {
            size += mdia.box_size();
        }
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for TrakBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0))?; // Current cursor position.
        let mut trak = TrakBox::new();

        let start = 0u64;
        while start < size {
            // Get box header.
            let header = read_box_header(reader, start)?;
            let BoxHeader{ name, size: s } = header;

            match name {
                BoxType::TkhdBox => {
                    let tkhd = TkhdBox::read_box(reader, s)?;
                    trak.tkhd = Some(tkhd);
                }
                BoxType::EdtsBox => {
                    let edts = EdtsBox::read_box(reader, s)?;
                    trak.edts = Some(edts);
                }
                BoxType::MdiaBox => {
                    let mdia = MdiaBox::read_box(reader, s)?;
                    trak.mdia = Some(mdia);
                }
                _ => break
            }
        }
        skip_read(reader, current, size)?;

        Ok(trak)
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for TrakBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

        if let Some(tkhd) = &self.tkhd {
            tkhd.write_box(writer)?;
        }
        if let Some(edts) = &self.edts {
            edts.write_box(writer)?;
        }
        if let Some(mdia) = &self.mdia {
            mdia.write_box(writer)?;
        }

        Ok(size)
    }
}
