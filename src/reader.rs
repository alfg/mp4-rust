use std::io::{Seek, SeekFrom, Read};

use crate::{Result, Error, Mp4Sample};
use crate::atoms::*;

#[derive(Debug)]
pub struct Mp4Reader<R> {
    reader: R,
    pub ftyp: FtypBox,
    pub moov: Option<MoovBox>,
    size: u64,
}

impl<R: Read + Seek> Mp4Reader<R> {
    pub fn new(reader: R) -> Self {
        Mp4Reader {
            reader,
            ftyp: FtypBox::default(),
            moov: None,
            size: 0,
        }
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn read(&mut self, size: u64) -> Result<()> {
        let start = self.reader.seek(SeekFrom::Current(0))?;
        let mut current = start;
        while current < size {
            // Get box header.
            let header = BoxHeader::read(&mut self.reader)?;
            let BoxHeader{ name, size: s } = header;

            // Match and parse the atom boxes.
            match name {
                BoxType::FtypBox => {
                    let ftyp = FtypBox::read_box(&mut self.reader, s)?;
                    self.ftyp = ftyp;
                }
                BoxType::FreeBox => {
                    skip_box(&mut self.reader, s)?;
                }
                BoxType::MdatBox => {
                    skip_box(&mut self.reader, s)?;
                }
                BoxType::MoovBox => {
                    let moov = MoovBox::read_box(&mut self.reader, s)?;
                    self.moov = Some(moov);
                }
                BoxType::MoofBox => {
                    skip_box(&mut self.reader, s)?;
                }
                _ => {
                    // XXX warn!()
                    skip_box(&mut self.reader, s)?;
                }
            }
            current = self.reader.seek(SeekFrom::Current(0))?;
        }
        self.size = current - start;
        Ok(())
    }

    pub fn track_count(&self) -> Result<u32> {
        if let Some(ref moov) = self.moov {
            Ok(moov.traks.len() as u32)
        } else {
            Err(Error::BoxNotFound(MoovBox::box_type()))
        }
    }

    pub fn sample_count(&self, track_id: u32) -> Result<u32> {
        if track_id == 0 {
            return Err(Error::TrakNotFound(track_id));
        }

        let moov = if let Some(ref moov) = self.moov {
            moov
        } else {
            return Err(Error::BoxNotFound(MoovBox::box_type()));
        };

        let trak = if let Some(trak) = moov.traks.get(track_id as usize - 1) {
            trak
        } else {
            return Err(Error::TrakNotFound(track_id));
        };

        trak.sample_count()
    }

    pub fn read_sample(
        &mut self,
        track_id: u32,
        sample_id: u32,
    ) -> Result<Option<Mp4Sample>> {
        if track_id == 0 {
            return Err(Error::TrakNotFound(track_id));
        }

        let moov = if let Some(ref moov) = self.moov {
            moov
        } else {
            return Err(Error::BoxNotFound(MoovBox::box_type()));
        };

        let trak = if let Some(trak) = moov.traks.get(track_id as usize - 1) {
            trak
        } else {
            return Err(Error::TrakNotFound(track_id));
        };

        trak.read_sample(&mut self.reader, sample_id)
    }
}
