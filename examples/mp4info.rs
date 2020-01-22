extern crate mp4;

use std::env;
use std::fs::File;
use std::any::Any;
use mp4::{FourCC, TrackType};

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        2 => {
            let filename = &args[1];
            let f = File::open(filename).unwrap();

            let bmff = mp4::read_mp4(f).unwrap();
            let moov = bmff.moov.unwrap();

            // Print results.
//            println!("{:?}", bmff.unwrap());
            println!("File:");
            println!("  file size:  {}",  bmff.size);
            println!("  brands:     {:?} {:?}\n",  bmff.ftyp.major_brand,  bmff.ftyp.compatible_brands);

            println!("Movie:");
            println!("  duration:   {:?}",  moov.mvhd.duration);
            println!("  timescale:  {:?}\n",  moov.mvhd.timescale);

            println!("Found {} Tracks", moov.traks.len());
            for trak in moov.traks.iter() {
                let tkhd = trak.tkhd.as_ref().unwrap();
                let mdia = trak.mdia.as_ref().unwrap();
                let hdlr = mdia.hdlr.as_ref().unwrap();
                let mdhd = mdia.mdhd.as_ref().unwrap();

                println!("  flags:    {:?}", tkhd.flags);
                println!("  id:       {:?}", tkhd.track_id);
                println!("  type:     {:?}", get_handler_type(hdlr.handler_type.value.as_ref()));
                println!("  duration: {:?}", tkhd.duration);
                println!("  language: {:?}", mdhd.language_string);
                println!("  width: {:?}", tkhd.width);
                println!("  height: {:?}\n", tkhd.height);
            }
        },
        _ => {
            println!("Usage: mp4info <filename>");
        }
    }
}

fn get_handler_type(handler: &str) -> TrackType {
    let mut typ: TrackType = TrackType::Unknown;
    match handler {
        "vide" => typ = TrackType::Video,
        "soun" => typ = TrackType::Audio,
        "meta" => typ = TrackType::Unknown,
        _ => (),
    }
    return typ;
}