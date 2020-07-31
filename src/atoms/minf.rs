use std::io::{Seek, SeekFrom, Read, Write};

use crate::*;
use crate::atoms::*;
use crate::atoms::{vmhd::VmhdBox, smhd::SmhdBox, stbl::StblBox};


#[derive(Debug, Default)]
pub struct MinfBox {
    pub vmhd: Option<VmhdBox>,
    pub smhd: Option<SmhdBox>,
    pub stbl: Option<StblBox>,
}

impl MinfBox {
    pub(crate) fn new() -> MinfBox {
        Default::default()
    }
}

impl Mp4Box for MinfBox {
    fn box_type() -> BoxType {
        BoxType::MinfBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        if let Some(ref vmhd) = self.vmhd {
            size += vmhd.box_size();
        }
        if let Some(ref smhd) = self.smhd {
            size += smhd.box_size();
        }
        if let Some(ref stbl) = self.stbl {
            size += stbl.box_size();
        }
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for MinfBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = get_box_start(reader)?;

        let mut minf = MinfBox::new();

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader{ name, size: s } = header;

            match name {
                BoxType::VmhdBox => {
                    let vmhd = VmhdBox::read_box(reader, s)?;
                    minf.vmhd = Some(vmhd);
                }
                BoxType::SmhdBox => {
                    let smhd = SmhdBox::read_box(reader, s)?;
                    minf.smhd = Some(smhd);
                }
                BoxType::DinfBox => {// XXX warn!()
                    skip_box(reader, s)?;
                }
                BoxType::StblBox => {
                    let stbl = StblBox::read_box(reader, s)?;
                    minf.stbl = Some(stbl);
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }

            current = reader.seek(SeekFrom::Current(0))?;
        }

        skip_read_to(reader, start + size)?;

        Ok(minf)
    }
}

impl<W: Write> WriteBox<&mut W> for MinfBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(Self::box_type(), size).write(writer)?;

        if let Some(ref vmhd) = self.vmhd {
            vmhd.write_box(writer)?;
        }
        if let Some(ref smhd) = self.smhd {
            smhd.write_box(writer)?;
        }
        if let Some(ref stbl) = self.stbl {
            stbl.write_box(writer)?;
        }

        Ok(size)
    }
}
