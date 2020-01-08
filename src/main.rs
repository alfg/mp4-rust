use std::io::prelude::*;
use std::io::{BufReader, Read, SeekFrom};
use std::fs::File;
use std::convert::TryInto;

// struct Box {
//     name: String,
//     size: u32,
//     offset: u32,
// }

fn main() -> std::io::Result<()> {

    // Using BufReader.
    let mut f = File::open("tears-of-steel-2s.mp4")?;
    let filesize = f.metadata().unwrap().len();
    println!("{:?}", filesize);
    let mut reader = BufReader::new(f);

    let mut offset = 0u64;

    while offset < filesize {

        // reader.seek(SeekFrom::Current(40 + 2872360));
        reader.seek(SeekFrom::Current(offset as i64));

        let mut buf = [0u8;8];
        let n = reader.read(&mut buf);

        let s = buf[0..4].try_into().unwrap();
        let size = u32::from_be_bytes(s);
        
        let t = buf[4..8].try_into().unwrap();
        let typ = match std::str::from_utf8(t) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };

        // Exit loop if size is 0.
        if size == 0 { break; }

        // println!("{}", buf.len());
        // println!("{:?}", buf);
        println!("{:?}", size);
        println!("{:?}", typ);

        // This will find all boxes, including nested boxes.
        // let mut offset = match size {
        //     1 => 4 + 4 + 8,
        //     _ => 4 + 4,
        // };
        // assert!(offset <= size);

        offset = (size - 8) as u64;
        println!("skip {:?}\n", offset);
    }

    println!("done");
    Ok(())
}