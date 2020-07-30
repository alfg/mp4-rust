use std::io::{Seek, SeekFrom, Read};
use std::convert::TryInto;
use bytes::Buf;

mod atoms;
use crate::atoms::*;

mod error;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum TrackType {
    Audio,
    Video,
    Metadata,
    Unknown,
}

#[derive(Debug)]
pub struct Sample<B> {
    pub start_time: u64,
    pub duration: u32,
    pub rendering_offset: u32,
    pub is_sync: bool,
    pub data: B,
}

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

    pub fn read_sample<B: Buf>(
        &mut self,
        track_id: u32,
        sample_id: u32,
    ) -> Result<Option<Sample<B>>> {
        if track_id == 0 {
            return Err(Error::TrakNotFound(track_id));
        }

        let moov = if let Some(moov) = &self.moov {
            moov
        } else {
            return Err(Error::BoxNotFound(MoovBox::box_type()));
        };

        let trak = if let Some(trak) = moov.traks.get(track_id as usize - 1) {
            trak
        } else {
            return Err(Error::TrakNotFound(track_id));
        };

        let _sample_offset = trak.sample_offset(sample_id)?;
        let _sample_size = trak.sample_size(sample_id)?;

        // TODO

        Ok(None)
    }
}
