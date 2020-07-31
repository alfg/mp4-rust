use std::io::{Seek, SeekFrom, Read, Write};

use crate::*;
use crate::atoms::*;
use crate::atoms::{
    stsd::StsdBox,
    stts::SttsBox,
    ctts::CttsBox,
    stss::StssBox,
    stsc::StscBox,
    stsz::StszBox,
    stco::StcoBox,
    co64::Co64Box,
};


#[derive(Debug, Default)]
pub struct StblBox {
    pub stsd: Option<StsdBox>,
    pub stts: Option<SttsBox>,
    pub ctts: Option<CttsBox>,
    pub stss: Option<StssBox>,
    pub stsc: Option<StscBox>,
    pub stsz: Option<StszBox>,
    pub stco: Option<StcoBox>,
    pub co64: Option<Co64Box>,
}

impl StblBox {
    pub(crate) fn new() -> StblBox {
        Default::default()
    }
}

impl Mp4Box for StblBox {
    fn box_type() -> BoxType {
        BoxType::StblBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        if let Some(ref stsd) = self.stsd {
            size += stsd.box_size();
        }
        if let Some(ref stts) = self.stts {
            size += stts.box_size();
        }
        if let Some(ref ctts) = self.ctts {
            size += ctts.box_size();
        }
        if let Some(ref stss) = self.stss {
            size += stss.box_size();
        }
        if let Some(ref stsc) = self.stsc {
            size += stsc.box_size();
        }
        if let Some(ref stsz) = self.stsz {
            size += stsz.box_size();
        }
        if let Some(ref stco) = self.stco {
            size += stco.box_size();
        }
        if let Some(ref co64) = self.co64 {
            size += co64.box_size();
        }
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for StblBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = get_box_start(reader)?;

        let mut stbl = StblBox::new();

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader{ name, size: s } = header;

            match name {
                BoxType::StsdBox => {
                    let stsd = StsdBox::read_box(reader, s)?;
                    stbl.stsd = Some(stsd);
                }
                BoxType::SttsBox => {
                    let stts = SttsBox::read_box(reader, s)?;
                    stbl.stts = Some(stts);
                }
                BoxType::CttsBox => {
                    let ctts = CttsBox::read_box(reader, s)?;
                    stbl.ctts = Some(ctts);
                }
                BoxType::StssBox => {
                    let stss = StssBox::read_box(reader, s)?;
                    stbl.stss = Some(stss);
                }
                BoxType::StscBox => {
                    let stsc = StscBox::read_box(reader, s)?;
                    stbl.stsc = Some(stsc);
                }
                BoxType::StszBox => {
                    let stsz = StszBox::read_box(reader, s)?;
                    stbl.stsz = Some(stsz);
                }
                BoxType::StcoBox => {
                    let stco = StcoBox::read_box(reader, s)?;
                    stbl.stco = Some(stco);
                }
                BoxType::Co64Box => {
                    let co64 = Co64Box::read_box(reader, s)?;
                    stbl.co64 = Some(co64);
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }
            current = reader.seek(SeekFrom::Current(0))?;
        }

        skip_read_to(reader, start + size)?;

        Ok(stbl)
    }
}

impl<W: Write> WriteBox<&mut W> for StblBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(Self::box_type(), size).write(writer)?;

        if let Some(ref stsd) = self.stsd {
            stsd.write_box(writer)?;
        }
        if let Some(ref stts) = self.stts {
            stts.write_box(writer)?;
        }
        if let Some(ref ctts) = self.ctts {
            ctts.write_box(writer)?;
        }
        if let Some(ref stss) = self.stss {
            stss.write_box(writer)?;
        }
        if let Some(ref stsc) = self.stsc {
            stsc.write_box(writer)?;
        }
        if let Some(ref stsz) = self.stsz {
            stsz.write_box(writer)?;
        }
        if let Some(ref stco) = self.stco {
            stco.write_box(writer)?;
        }
        if let Some(ref co64) = self.co64 {
            co64.write_box(writer)?;
        }

        Ok(size)
    }
}
