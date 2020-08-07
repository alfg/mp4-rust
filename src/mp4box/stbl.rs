use std::io::{Read, Seek, SeekFrom, Write};

use crate::mp4box::*;
use crate::mp4box::{
    co64::Co64Box, ctts::CttsBox, stco::StcoBox, stsc::StscBox, stsd::StsdBox, stss::StssBox,
    stsz::StszBox, stts::SttsBox,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StblBox {
    pub stsd: StsdBox,
    pub stts: SttsBox,
    pub ctts: Option<CttsBox>,
    pub stss: Option<StssBox>,
    pub stsc: StscBox,
    pub stsz: StszBox,
    pub stco: Option<StcoBox>,
    pub co64: Option<Co64Box>,
}

impl Mp4Box for StblBox {
    fn box_type() -> BoxType {
        BoxType::StblBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        size += self.stsd.box_size();
        size += self.stts.box_size();
        if let Some(ref ctts) = self.ctts {
            size += ctts.box_size();
        }
        if let Some(ref stss) = self.stss {
            size += stss.box_size();
        }
        size += self.stsc.box_size();
        size += self.stsz.box_size();
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
        let start = box_start(reader)?;

        let mut stsd = None;
        let mut stts = None;
        let mut ctts = None;
        let mut stss = None;
        let mut stsc = None;
        let mut stsz = None;
        let mut stco = None;
        let mut co64 = None;

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader { name, size: s } = header;

            match name {
                BoxType::StsdBox => {
                    stsd = Some(StsdBox::read_box(reader, s)?);
                }
                BoxType::SttsBox => {
                    stts = Some(SttsBox::read_box(reader, s)?);
                }
                BoxType::CttsBox => {
                    ctts = Some(CttsBox::read_box(reader, s)?);
                }
                BoxType::StssBox => {
                    stss = Some(StssBox::read_box(reader, s)?);
                }
                BoxType::StscBox => {
                    stsc = Some(StscBox::read_box(reader, s)?);
                }
                BoxType::StszBox => {
                    stsz = Some(StszBox::read_box(reader, s)?);
                }
                BoxType::StcoBox => {
                    stco = Some(StcoBox::read_box(reader, s)?);
                }
                BoxType::Co64Box => {
                    co64 = Some(Co64Box::read_box(reader, s)?);
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }
            current = reader.seek(SeekFrom::Current(0))?;
        }

        if stsd.is_none() {
            return Err(Error::BoxNotFound(BoxType::StsdBox));
        }
        if stts.is_none() {
            return Err(Error::BoxNotFound(BoxType::SttsBox));
        }
        if stsc.is_none() {
            return Err(Error::BoxNotFound(BoxType::StscBox));
        }
        if stsz.is_none() {
            return Err(Error::BoxNotFound(BoxType::StszBox));
        }
        if stco.is_none() && co64.is_none() {
            return Err(Error::Box2NotFound(BoxType::StcoBox, BoxType::Co64Box));
        }

        skip_bytes_to(reader, start + size)?;

        Ok(StblBox {
            stsd: stsd.unwrap(),
            stts: stts.unwrap(),
            ctts: ctts,
            stss: stss,
            stsc: stsc.unwrap(),
            stsz: stsz.unwrap(),
            stco: stco,
            co64: co64,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for StblBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(Self::box_type(), size).write(writer)?;

        self.stsd.write_box(writer)?;
        self.stts.write_box(writer)?;
        if let Some(ref ctts) = self.ctts {
            ctts.write_box(writer)?;
        }
        if let Some(ref stss) = self.stss {
            stss.write_box(writer)?;
        }
        self.stsc.write_box(writer)?;
        self.stsz.write_box(writer)?;
        if let Some(ref stco) = self.stco {
            stco.write_box(writer)?;
        }
        if let Some(ref co64) = self.co64 {
            co64.write_box(writer)?;
        }

        Ok(size)
    }
}
