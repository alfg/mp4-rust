use std::io::prelude::*;
use std::io::{BufReader, Read, SeekFrom};
use std::fs::File;
use std::fmt;
use std::convert::TryInto;
use byteorder::{BigEndian, ReadBytesExt};

#[derive(Debug)]
struct Box {
    name: String,
    size: u32,
    offset: u32,
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

    let mut offset = 0u64;
    while offset < filesize {

        // Seek to offset.
        let _r = reader.seek(SeekFrom::Current(offset as i64));

        // Create and read to buf.
        let mut buf = [0u8;8];
        let _n = reader.read(&mut buf);

        // Get size.
        let s = buf[0..4].try_into().unwrap();
        let size = u32::from_be_bytes(s);

        // Exit loop if size is 0.
        if size == 0 { break; }
        
        // Get box type string.
        let t = buf[4..8].try_into().unwrap();
        let typ = match std::str::from_utf8(t) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };

        match typ.as_ref() {
            "ftyp" =>  parse_ftyp_box(&mut reader, 0, size),
            _ => (),
        };


        // Make Box struct and add to vector.
        let b = Box{
            name: typ.try_into().unwrap(),
            size: size,
            offset: offset as u32,
        };
        boxes.push(b);

        // This will find all boxes, including nested boxes.
        // let mut offset = match size {
        //     1 => 4 + 4 + 8,
        //     _ => 4 + 4,
        // };
        // assert!(offset <= size);

        // Increment offset.
        offset = (size - 8) as u64;
    }

    // Print results.
    for b in boxes {
        println!("{:?}", b);

    }

    // Done.
    println!("done");
    Ok(())
}

fn parse_ftyp_box(f: &mut BufReader<File>, offset: u64, size: u32) {
    println!("found ftyp");

    let major = f.read_u32::<byteorder::BigEndian>().unwrap();
    let minor = f.read_u32::<byteorder::BigEndian>().unwrap();
    let brand_count = (size - 16) / 4; // header + major + minor

    println!("{}", brand_count);

    let mut brands = Vec::new();
    for _ in 0..brand_count {
        let b = f.read_u32::<byteorder::BigEndian>().unwrap();
        brands.push(From::from(b));
    }

    let ftyp = FtypBox {
        major_brand: From::from(major),
        minor_version: minor,
        compatible_brands: brands,
    };
    println!("{:?}", ftyp);

    println!("end ftyp");
    // Ok(FtypBox {
    //     major_brand: major,
    //     minor_version: minor,
    // })
}
