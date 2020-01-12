use std::io::prelude::*;
use std::io::{BufReader, Read, SeekFrom};
use std::fs::File;
use std::fmt;
use std::convert::TryInto;
use byteorder::{ReadBytesExt};

const HEADER_SIZE: u32 = 8;

#[derive(Debug)]
pub enum Error {
    InvalidData(&'static str),
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
struct BoxHeader {
    name: String,
    size: u64,
    offset: u64,
}

#[derive(Debug)]
struct FtypBox {
    major_brand: FourCC,
    minor_version: u32,
    compatible_brands: Vec<FourCC>,
}

#[derive(Default, PartialEq, Clone)]
pub struct FourCC {
    pub value: String
}

impl From<u32> for FourCC {
    fn from(number: u32) -> FourCC {
        let mut box_chars = Vec::new();
        for x in 0..4 {
            let c = (number >> (x * 8) & 0x0000_00FF) as u8;
            box_chars.push(c);
        }
        box_chars.reverse();

        let box_string = match String::from_utf8(box_chars) {
            Ok(t) => t,
            _ => String::from("null"), // error to retrieve fourcc
        };

        FourCC {
            value: box_string
        }
    }
}

impl fmt::Debug for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

fn main() -> std::io::Result<()> {

    // Using BufReader.
    let f = File::open("tears-of-steel-2s.mp4")?;
    let filesize = f.metadata().unwrap().len();
    let mut reader = BufReader::new(f);
    let mut boxes = Vec::new();

    let mut start = 0u64;
    while start < filesize {

        // Seek to offset.
        let _r = reader.seek(SeekFrom::Current(start as i64));

        // Create and read to buf.
        let mut buf = [0u8;8]; // 8 bytes for box header.
        let _n = reader.read(&mut buf);

        // Get size.
        let s = buf[0..4].try_into().unwrap();
        let size = u32::from_be_bytes(s);

        // Exit loop if size is 0.
        if size == 0 { break; }
        
        // Get box type string.
        // println!("{:?}", buf);
        let t = buf[4..8].try_into().unwrap();
        let typ = match std::str::from_utf8(t) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };

        let offset = match size {
            1 => 4 + 4 + 8,
            _ => 4 + 4,
        };

        // println!("{:?}", typ);
        // println!("{:?}", size);

        // Match and parse the filetype.
        match typ {
            "ftyp" => {
                let o = parse_ftyp_box(&mut reader, 0, size);
                println!("{:?}", o.unwrap());
                // start += (size - HEADER_SIZE) as u64;
                // TODO: Add to context.
            }
            "free" => {
                start = 0;
            }
            "mdat" => {
                start = (size - HEADER_SIZE) as u64;
            }
            "moov" => {
                start = (size - HEADER_SIZE) as u64;
            }
            "moof" => {
                start = (size - HEADER_SIZE) as u64;
            }
            _ => break
        };


        // Make Box struct and add to vector.
        let b = BoxHeader{
            name: typ.try_into().unwrap(),
            size: size as u64,
            offset: offset as u64,
        };
        boxes.push(b);
    }

    // Print results.
    for b in boxes {
        println!("{:?}", b);

    }

    // Done.
    println!("done");
    Ok(())
}

fn parse_ftyp_box(f: &mut BufReader<File>, _offset: u64, size: u32) -> Result<FtypBox> {
    let major = f.read_u32::<byteorder::BigEndian>().unwrap();
    let minor = f.read_u32::<byteorder::BigEndian>().unwrap();
    if size % 4 != 0 {
        return Err(Error::InvalidData("invalid ftyp size"));
    }
    let brand_count = (size - 16) / 4; // header + major + minor

    let mut brands = Vec::new();
    for _ in 0..brand_count {
        let b = f.read_u32::<byteorder::BigEndian>().unwrap();
        brands.push(From::from(b));
    }

    Ok(FtypBox {
        major_brand: From::from(major),
        minor_version: minor,
        compatible_brands: brands,
    })
}
