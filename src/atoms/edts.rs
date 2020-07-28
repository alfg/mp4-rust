use std::io::{BufReader, SeekFrom, Seek, Read, BufWriter, Write};

use crate::*;
use crate::atoms::elst::ElstBox;


#[derive(Debug, Default)]
pub struct EdtsBox {
    pub elst: Option<ElstBox>,
}

impl EdtsBox {
    pub(crate) fn new() -> EdtsBox {
        Default::default()
    }
}

impl Mp4Box for EdtsBox {
    fn box_type(&self) -> BoxType {
        BoxType::EdtsBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        if let Some(elst) = &self.elst {
            size += elst.box_size();
        }
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for EdtsBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0))?; // Current cursor position.
        let mut edts = EdtsBox::new();

        let start = 0u64;
        while start < size {
            // Get box header.
            let header = read_box_header(reader, start)?;
            let BoxHeader{ name, size: s } = header;

            match name {
                BoxType::ElstBox => {
                    let elst = ElstBox::read_box(reader, s)?;
                    edts.elst = Some(elst);
                }
                _ => break
            }
        }
        skip_read(reader, current, size)?;

        Ok(edts)
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for EdtsBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

        if let Some(elst) = &self.elst {
            elst.write_box(writer)?;
        }

        Ok(size)
    }
}
