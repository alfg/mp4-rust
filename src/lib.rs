use std::io::{BufReader, Read, SeekFrom, Seek};
use std::fs::File;
use std::convert::TryInto;

mod atoms;
use crate::atoms::*;

mod error;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

// XXX if box has largesize
const HEADER_SIZE: u64 = 8;

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

#[derive(Debug, Clone, Copy)]
struct BoxHeader {
    name: BoxType,
    size: u64,
}

impl BoxHeader {
    fn new(name: BoxType, size: u64) -> Self {
        Self { name, size }
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

    let mut start = 0u64;
    while start < filesize {

        // Get box header.
        let header = read_box_header(&mut reader, start)?;
        let BoxHeader{ name, size } = header;

        // Match and parse the atom boxes.
        match name {
            BoxType::FtypBox => {
                let ftyp = FtypBox::read_box(&mut reader, size)?;
                bmff.ftyp = ftyp;
            }
            BoxType::FreeBox => {
                start = 0;
            }
            BoxType::MdatBox => {
                start = size - HEADER_SIZE;
            }
            BoxType::MoovBox => {
                let moov = MoovBox::read_box(&mut reader, size)?;
                bmff.moov = Some(moov);
            }
            BoxType::MoofBox => {
                start = size - HEADER_SIZE;
            }
            _ => {
                // Skip over unsupported boxes, but stop if the size is zero,
                // meaning the last box has been reached.
                if size == 0 {
                    break;
                } else {
                    start = size - HEADER_SIZE;
                }
            }
        };
    }
    Ok(bmff)
}

// TODO: if size is 0, then this box is the last one in the file
fn read_box_header<R: Read + Seek>(reader: &mut BufReader<R>, start: u64) -> Result<BoxHeader> {
    // Seek to offset.
    let _r = reader.seek(SeekFrom::Current(start as i64));

    // Create and read to buf.
    let mut buf = [0u8;8]; // 8 bytes for box header.
    reader.read(&mut buf)?;

    // Get size.
    let s = buf[0..4].try_into().unwrap();
    let size = u32::from_be_bytes(s);
    
    // Get box type string.
    let t = buf[4..8].try_into().unwrap();
    let typ = u32::from_be_bytes(t);

    // Get largesize if size is 1
    if size == 1 {
        reader.read(&mut buf)?;
        let s = buf.try_into().unwrap();
        let largesize = u64::from_be_bytes(s);

        Ok(BoxHeader {
            name: BoxType::from(typ),
            size: largesize,
        })
    } else {
        Ok(BoxHeader {
            name: BoxType::from(typ),
            size: size as u64,
        })
    }
}


