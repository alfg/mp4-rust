use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::path::Path;

use mp4::Result;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: mp4copy <source file> <target file>");
        std::process::exit(1);
    }

    if let Err(err) = copy(&args[1], &args[2]) {
        let _ = writeln!(io::stderr(), "{}", err);
    }
}

fn copy<P: AsRef<Path>>(src_filename: &P, _dst_filename: &P) -> Result<()> {
    let src_file = File::open(src_filename)?;
    let size = src_file.metadata()?.len();
    let reader = BufReader::new(src_file);

    let mut mp4 = mp4::Mp4Reader::new(reader);
    mp4.read(size)?;

    for tix in 0..mp4.track_count()? {
        let track_id = tix + 1;
        let sample_count = mp4.sample_count(track_id)?;
        for six in 0..sample_count {
            let sample_id = six + 1;
            let sample = mp4.read_sample(track_id, sample_id)?.unwrap();
            println!("sample_id: {}, {}", sample_id, sample);
        }
    }

    Ok(())
}
