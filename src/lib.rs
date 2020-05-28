extern crate byteorder;

use std::io::prelude::*;
use std::io::{BufReader, Read, SeekFrom};
use std::fs::File;
use std::convert::TryInto;

mod atoms;
use crate::atoms::*;

const HEADER_SIZE: u32 = 8;

#[derive(Debug)]
pub enum Error {
    InvalidData(&'static str),
}

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
    offset: u64,
}

pub fn read_mp4(f: File) -> Result<BMFF> {

    // Open file and read boxes.
    let bmff = read_boxes(f).unwrap();

    Ok(bmff)
}

fn read_boxes(f: File) -> Result<BMFF> {
    let filesize = f.metadata().unwrap().len();
    let mut reader = BufReader::new(f);
    let mut bmff = BMFF::new();
    bmff.size  =  filesize;

    let mut start = 0u64;
    while start < filesize {

        // Get box header.
        let header = read_box_header(&mut reader, start).unwrap();
        let BoxHeader{ name, size, offset: _ } = header;

        // Match and parse the atom boxes.
        match name {
            BoxType::FtypBox => {
                let ftyp = parse_ftyp_box(&mut reader, 0, size as u32).unwrap();
                bmff.ftyp = ftyp;
            }
            BoxType::FreeBox => {
                start = 0;
            }
            BoxType::MdatBox => {
                start = (size as u32 - HEADER_SIZE) as u64;
            }
            BoxType::MoovBox => {
                let moov = parse_moov_box(&mut reader, 0, size as u32).unwrap();
                bmff.moov = Some(moov);
            }
            BoxType::MoofBox => {
                start = (size as u32 - HEADER_SIZE) as u64;
            }
            _ => {
                // Skip over unsupported boxes, but stop if the size is zero,
                // meaning the last box has been reached.
                if size == 0 {
                    break;
                } else {
                    start = (size as u32 - HEADER_SIZE) as u64;
                }
            }
        };
    }
    Ok(bmff)
}

fn read_box_header(reader: &mut BufReader<File>, start: u64) -> Result<BoxHeader> {
    // Seek to offset.
    let _r = reader.seek(SeekFrom::Current(start as i64));

    // Create and read to buf.
    let mut buf = [0u8;8]; // 8 bytes for box header.
    reader.read(&mut buf).unwrap();

    // Get size.
    let s = buf[0..4].try_into().unwrap();
    let size = u32::from_be_bytes(s);

    // TODO: Err if size is 0.
    // if size == 0 { break; }
    
    // Get box type string.
    let t = buf[4..8].try_into().unwrap();
    let typ = u32::from_be_bytes(t);

    let offset = match size {
        1 => 4 + 4 + 8,
        _ => 4 + 4,
    };

    Ok(BoxHeader {
        name: BoxType::from(typ),
        size: size as u64,
        offset: offset as u64,
    })
}


