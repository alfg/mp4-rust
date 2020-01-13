extern crate mp4;

use std::env;
use std::fs::File;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        2 => {
            let filename = &args[1];
            let f = File::open(filename).unwrap();

            let bmff = mp4::read_mp4(f);

            // Print results.
            println!("{:?}", bmff.unwrap());
        },
        _ => {
            println!("Usage: mp4info <filename>");
        }
    }

}