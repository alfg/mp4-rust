extern crate mp4;

use std::env;
use std::fs::File;
use std::any::Any;
use std::borrow::Borrow;
use std::fmt::Debug;
use mp4::{TrackType};

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        2 => {
            let filename = &args[1];
            let f = File::open(filename).unwrap();

            let bmff = mp4::read_mp4(f).unwrap();
            let moov = bmff.moov.unwrap();

            // Print results.
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
                let stts= mdia.minf.as_ref().unwrap()
                    .stbl.as_ref().unwrap()
                    .stts.as_ref().unwrap();

                println!("Track: {:?}", tkhd.track_id);
                println!("  flags:    {:?}", tkhd.flags);
                println!("  id:       {:?}", tkhd.track_id);
                println!("  type:     {:?}", get_handler_type(hdlr.handler_type.value.as_ref()));
                println!("  duration: {:?}", tkhd.duration);
                println!("  language: {:?}", mdhd.language_string);

                println!("  media:");
                println!("    sample count: {:?}", stts.sample_counts[0]);
                println!("    timescale:    {:?}", mdhd.timescale);
                println!("    duration:     {:?} (media timescale units)", mdhd.duration);
                println!("    duration:     {:?} (ms)", getDurationMS(mdhd.duration, mdhd.timescale));
                if tkhd.width != 0 && tkhd.height != 0 {
                    println!("    width:    {:?}", tkhd.width);
                    println!("    height:   {:?}", tkhd.height);
                }
                if get_handler_type(hdlr.handler_type.value.as_ref()) == TrackType::Video {
                    println!("    frame rate: (computed): {:?}", getFramerate(&stts.sample_counts, mdhd.duration, mdhd.timescale));
                }
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

fn getDurationMS(duration: u32, timescale: u32) -> String {
    let ms = (duration as f64 / timescale as f64) * 1000.0;
    return format!("{:.2}", ms.floor());
}

fn getFramerate(sample_counts: &Vec<u32>, duration: u32, timescale: u32) -> String {
    let sc = (sample_counts[0] as f64) * 1000.0;
    let ms = (duration as f64 / timescale as f64) * 1000.0;
    return format!("{:.2}", sc / ms.floor());
}