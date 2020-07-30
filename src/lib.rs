use std::io::{BufReader, SeekFrom, Seek};
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
    fn new() -> BMFF {
        Default::default()
    }
}

pub fn read_mp4(f: File) -> Result<BMFF> {

    // Open file and read boxes.
    let bmff = read_boxes(f)?;

    Ok(bmff)
}

fn read_boxes(f: File) -> Result<BMFF> {
    let filesize = f.metadata()?.len();
    let mut reader = BufReader::new(f);
    let mut bmff = BMFF::new();
    bmff.size  =  filesize;

    let mut current = reader.seek(SeekFrom::Current(0))?;
    while current < filesize {
        // Get box header.
        let header = BoxHeader::read(&mut reader)?;
        let BoxHeader{ name, size } = header;

        // Match and parse the atom boxes.
        match name {
            BoxType::FtypBox => {
                let ftyp = FtypBox::read_box(&mut reader, size)?;
                bmff.ftyp = ftyp;
            }
            BoxType::FreeBox => {
                skip_box(&mut reader, size)?;
            }
            BoxType::MdatBox => {
                skip_box(&mut reader, size)?;
            }
            BoxType::MoovBox => {
                let moov = MoovBox::read_box(&mut reader, size)?;
                bmff.moov = Some(moov);
            }
            BoxType::MoofBox => {
                skip_box(&mut reader, size)?;
            }
            _ => {
                // XXX warn!()
                skip_box(&mut reader, size)?;
            }
        }
        current = reader.seek(SeekFrom::Current(0))?;
    }
    Ok(bmff)
}
