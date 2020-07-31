use std::fmt;
use std::io::{Seek, SeekFrom, Read};
use std::convert::TryInto;

pub use bytes::Bytes;

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
pub struct Mp4Sample {
    pub start_time: u64,
    pub duration: u32,
    pub rendering_offset: i32,
    pub is_sync: bool,
    pub bytes: Bytes,
}

impl PartialEq for Mp4Sample {
    fn eq(&self, other: &Self) -> bool {
        self.start_time == other.start_time
            && self.duration == other.duration
            && self.rendering_offset == other.rendering_offset
            && self.is_sync == other.is_sync
            && self.bytes.len() == other.bytes.len() // XXX for easy check
    }
}

impl fmt::Display for Mp4Sample {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "start_time {}, duration {}, rendering_offset {}, is_sync {}, length {}",
               self.start_time, self.duration, self.rendering_offset, self.is_sync,
               self.bytes.len())
    }
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
