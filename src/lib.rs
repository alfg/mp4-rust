use std::io::{BufReader, Read, Seek, SeekFrom};
use std::fs::File;
use std::convert::TryInto;

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

#[derive(Debug, Default)]
pub struct BMFF {
    pub ftyp: FtypBox,
    pub moov: Option<MoovBox>,
    pub size: u64,
}

impl BMFF {
    pub fn new() -> BMFF {
        Default::default()
    }

    pub fn read_from_file(f: File) -> Result<BMFF> {
        let size = f.metadata()?.len();
        let mut reader = BufReader::new(f);

        let mut bmff = BMFF::new();
        bmff.size = bmff.read(&mut reader, size)?;

        Ok(bmff)
    }

    pub fn read<R: Read + Seek>(
        &mut self,
        reader: &mut BufReader<R>,
        size: u64
    ) -> Result<u64> {
        let start = reader.seek(SeekFrom::Current(0))?;
        let mut current = start;
        while current < size {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader{ name, size: s } = header;

            // Match and parse the atom boxes.
            match name {
                BoxType::FtypBox => {
                    let ftyp = FtypBox::read_box(reader, s)?;
                    self.ftyp = ftyp;
                }
                BoxType::FreeBox => {
                    skip_box(reader, s)?;
                }
                BoxType::MdatBox => {
                    skip_box(reader, s)?;
                }
                BoxType::MoovBox => {
                    let moov = MoovBox::read_box(reader, s)?;
                    self.moov = Some(moov);
                }
                BoxType::MoofBox => {
                    skip_box(reader, s)?;
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }
            current = reader.seek(SeekFrom::Current(0))?;
        }
        Ok(current - start)
    }
}
