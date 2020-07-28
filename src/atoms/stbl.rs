use std::io::{BufReader, Seek, Read, BufWriter, Write};

use crate::*;
use crate::atoms::{stts::SttsBox, stsd::StsdBox};


#[derive(Debug, Default)]
pub struct StblBox {
    pub stts: Option<SttsBox>,
    pub stsd: Option<StsdBox>,
}

impl StblBox {
    pub(crate) fn new() -> StblBox {
        Default::default()
    }
}

impl Mp4Box for StblBox {
    fn box_type(&self) -> BoxType {
        BoxType::StblBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        if let Some(stts) = &self.stts {
            size += stts.box_size();
        }
        if let Some(stsd) = &self.stsd {
            size += stsd.box_size();
        }
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for StblBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let mut stbl = StblBox::new();

        let start = 0u64;
        while start < size {
            // Get box header.
            let header = read_box_header(reader, start)?;
            let BoxHeader{ name, size: s } = header;

            match name {
                BoxType::SttsBox => {
                    let stts = SttsBox::read_box(reader, s)?;
                    stbl.stts = Some(stts);
                }
                BoxType::StsdBox => {
                    let stsd = StsdBox::read_box(reader, s)?;
                    stbl.stsd = Some(stsd);
                }
                _ => break
            }
        }
        Ok(stbl)
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for StblBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

        if let Some(stts) = &self.stts {
            stts.write_box(writer)?;
        }
        if let Some(stsd) = &self.stsd {
            stsd.write_box(writer)?;
        }

        Ok(size)
    }
}
