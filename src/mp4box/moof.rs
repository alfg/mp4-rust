use std::io::{Read, Seek, SeekFrom, Write};

use crate::mp4box::*;
use crate::mp4box::{mfhd::MfhdBox, traf::TrafBox};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MoofBox {
    pub mfhd: MfhdBox,
    pub trafs: Vec<TrafBox>,
}

impl MoofBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::MoofBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE + self.mfhd.box_size();
        for traf in self.trafs.iter() {
            size += traf.box_size();
        }
        size
    }
}

impl Mp4Box for MoofBox {
    fn box_type(&self) -> BoxType {
        return self.get_type();
    }

    fn box_size(&self) -> u64 {
        return self.get_size();
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for MoofBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut mfhd = None;
        let mut trafs = Vec::new();

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader { name, size: s } = header;

            match name {
                BoxType::MfhdBox => {
                    mfhd = Some(MfhdBox::read_box(reader, s)?);
                }
                BoxType::TrafBox => {
                    let traf = TrafBox::read_box(reader, s)?;
                    trafs.push(traf);
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }
            current = reader.seek(SeekFrom::Current(0))?;
        }

        if mfhd.is_none() {
            return Err(Error::BoxNotFound(BoxType::MfhdBox));
        }

        skip_bytes_to(reader, start + size)?;

        Ok(MoofBox {
            mfhd: mfhd.unwrap(),
            trafs,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for MoofBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        self.mfhd.write_box(writer)?;
        for traf in self.trafs.iter() {
            traf.write_box(writer)?;
        }
        Ok(0)
    }
}
