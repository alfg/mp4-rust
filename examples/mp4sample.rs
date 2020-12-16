use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::path::Path;

use mp4::{Result};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: mp4sample <filename>");
        std::process::exit(1);
    }

    if let Err(err) = samples(&args[1]) {
        let _ = writeln!(io::stderr(), "{}", err);
    }
}

fn samples<P: AsRef<Path>>(filename: &P) -> Result<()> {
    let f = File::open(filename)?;
    let size = f.metadata()?.len();
    let reader = BufReader::new(f);

    let mut mp4 = mp4::Mp4Reader::read_header(reader, size)?;

    for track_idx in 0..mp4.tracks().len() {
        let track_id = track_idx as u32 + 1;
        let sample_count = mp4.sample_count(track_id).unwrap();

        for sample_idx in 0..sample_count {
            let sample_id = sample_idx + 1;
            let sample = mp4.read_sample(track_id, sample_id);

            if let Some(ref samp) = sample.unwrap() {
                println!("[{}] start_time={} duration={} rendering_offset={} size={} is_sync={}",
                  sample_id,
                  samp.start_time,
                  samp.duration,
                  samp.rendering_offset,
                  samp.bytes.len(),
                  samp.is_sync,
                );
            }
        }
    }
    Ok(())
}
