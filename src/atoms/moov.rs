use std::io::{BufReader, Seek, Read, BufWriter, Write};

use crate::*;
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
    fn box_type(&self) -> BoxType {
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

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for MoovBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let mut moov = MoovBox::new();

        let mut start = 0u64;
        while start < size {

            // Get box header.
            let header = read_box_header(reader, start)?;
            let BoxHeader{ name, size: s } = header;

            match name {
                BoxType::MvhdBox => {
                    moov.mvhd = MvhdBox::read_box(reader, s)?;
                }
                BoxType::TrakBox => {
                    let trak = TrakBox::read_box(reader, s)?;
                    moov.traks.push(trak);
                }
                BoxType::UdtaBox => {
                    start = s - HEADER_SIZE;
                }
                _ => break
            }
        }
        Ok(moov)
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for MoovBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

        self.mvhd.write_box(writer)?;
        for trak in self.traks.iter() {
            trak.write_box(writer)?;
        }
        Ok(0)
    }
}
