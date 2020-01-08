use std::io::prelude::*;
use std::io::{BufReader, Read, SeekFrom};
use std::fs::File;
use std::convert::TryInto;

#[derive(Debug)]
struct Box {
    name: String,
    size: u32,
    offset: u32,
}

fn main() -> std::io::Result<()> {

    // Using BufReader.
    let f = File::open("tears-of-steel-2s.mp4")?;
    let filesize = f.metadata().unwrap().len();
    let mut reader = BufReader::new(f);
    let mut v = Vec::new();

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

        // Make Box struct and add to vector.
        let b = Box{
            name: typ.try_into().expect("asdf"),
            size: size,
            offset: offset as u32,
        };
        v.push(b);

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
    for a in v {
        println!("{:?}", a);
    }

    // Done.
    println!("done");
    Ok(())
}