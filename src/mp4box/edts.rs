use std::io::{Read, Seek, Write};

use crate::mp4box::elst::ElstBox;
use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EdtsBox {
    pub elst: Option<ElstBox>,
}

impl EdtsBox {
    pub(crate) fn new() -> EdtsBox {
        Default::default()
    }
}

impl Mp4Box for EdtsBox {
    fn box_type() -> BoxType {
        BoxType::EdtsBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        if let Some(ref elst) = self.elst {
            size += elst.box_size();
        }
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for EdtsBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut edts = EdtsBox::new();

        let header = BoxHeader::read(reader)?;
        let BoxHeader { name, size: s } = header;

        match name {
            BoxType::ElstBox => {
                let elst = ElstBox::read_box(reader, s)?;
                edts.elst = Some(elst);
            }
            _ => {}
        }

        skip_bytes_to(reader, start + size)?;

        Ok(edts)
    }
}

impl<W: Write> WriteBox<&mut W> for EdtsBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(Self::box_type(), size).write(writer)?;

        if let Some(ref elst) = self.elst {
            elst.write_box(writer)?;
        }

        Ok(size)
    }
}
