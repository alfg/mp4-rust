use std::io::{Seek, SeekFrom, Read, Write};

use crate::*;
use crate::atoms::*;
use crate::atoms::{mvhd::MvhdBox, trak::TrakBox};


#[derive(Debug, Default)]
pub struct MoovBox {
    pub mvhd: MvhdBox,
    pub traks: Vec<TrakBox>,
}

impl MoovBox {
    pub(crate) fn new() -> MoovBox {
        Default::default()
    }
}

impl Mp4Box for MoovBox {
    fn box_type() -> BoxType {
        BoxType::MoovBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE + self.mvhd.box_size();
        for trak in self.traks.iter() {
            size += trak.box_size();
        }
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for MoovBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = get_box_start(reader)?;

        let mut moov = MoovBox::new();

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader{ name, size: s } = header;

            match name {
                BoxType::MvhdBox => {
                    moov.mvhd = MvhdBox::read_box(reader, s)?;
                }
                BoxType::TrakBox => {
                    let mut trak = TrakBox::read_box(reader, s)?;
                    trak.id = moov.traks.len() as u32 + 1;
                    moov.traks.push(trak);
                }
                BoxType::UdtaBox => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }

            current = reader.seek(SeekFrom::Current(0))?;
        }

        skip_read_to(reader, start + size)?;

        Ok(moov)
    }
}

impl<W: Write> WriteBox<&mut W> for MoovBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(Self::box_type(), size).write(writer)?;

        self.mvhd.write_box(writer)?;
        for trak in self.traks.iter() {
            trak.write_box(writer)?;
        }
        Ok(0)
    }
}
