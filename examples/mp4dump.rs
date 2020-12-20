use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::path::Path;

use mp4::{Mp4Box, Result};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: mp4dump <filename>");
        std::process::exit(1);
    }

    if let Err(err) = dump(&args[1]) {
        let _ = writeln!(io::stderr(), "{}", err);
    }
}

fn dump<P: AsRef<Path>>(filename: &P) -> Result<()> {
    let f = File::open(filename)?;
    let boxes = get_boxes(f)?;

    // print out boxes
    for b in boxes.iter() {
        for _ in 0..b.indent {
            print!("  ");
        }
        println!("[{}] size={} {}", b.name, b.size, b.summary);
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Box {
    name: String,
    size: u64,
    summary: String,
    indent: u32,
}

fn get_boxes(file: File) -> Result<Vec<Box>> {
    let size = file.metadata()?.len();
    let reader = BufReader::new(file);
    let mp4 = mp4::Mp4Reader::read_header(reader, size)?;

    // collect known boxes
    let mut boxes = Vec::new();

    // ftyp, moov, mvhd
    boxes.push(build_box(&mp4.ftyp, 0));
    boxes.push(build_box(&mp4.moov, 0));
    boxes.push(build_box(&mp4.moov.mvhd, 1));

    if let Some(ref mvex) = &mp4.moov.mvex {
        boxes.push(build_box(mvex, 1));
        boxes.push(build_box(&mvex.mehd, 2));
        boxes.push(build_box(&mvex.trex, 2));
    }

    // trak.
    for track in mp4.tracks().iter() {
        boxes.push(build_box(&track.trak, 1));
        boxes.push(build_box(&track.trak.tkhd, 2));
        if let Some(ref edts) = track.trak.edts {
            boxes.push(build_box(edts, 2));
            if let Some(ref elst) = edts.elst {
                boxes.push(build_box(elst, 3));
            }
        }

        // trak.mdia
        let mdia = &track.trak.mdia;
        boxes.push(build_box(mdia, 2));
        boxes.push(build_box(&mdia.mdhd, 3));
        boxes.push(build_box(&mdia.hdlr, 3));
        boxes.push(build_box(&mdia.minf, 3));

        // trak.mdia.minf
        let minf = &track.trak.mdia.minf;
        boxes.push(descr_box("minf", 3));
        if let Some(ref vmhd) = &minf.vmhd {
            boxes.push(build_box(vmhd, 4));
        }
        if let Some(ref smhd) = &minf.smhd {
            boxes.push(build_box(smhd, 4));
        }

        // trak.mdia.minf.stbl
        let stbl = &track.trak.mdia.minf.stbl;
        boxes.push(build_box(stbl, 4));
        boxes.push(build_box(&stbl.stsd, 5));
        if let Some(ref avc1) = &stbl.stsd.avc1 {
            boxes.push(build_box(avc1, 6));
        }
        if let Some(ref hev1) = &stbl.stsd.hev1 {
            boxes.push(build_box(hev1, 6));
        }
        if let Some(ref mp4a) = &stbl.stsd.mp4a {
            boxes.push(build_box(mp4a, 6));
        }
        boxes.push(build_box(&stbl.stts, 5));
        if let Some(ref ctts) = &stbl.ctts {
            boxes.push(build_box(ctts, 5));
        }
        if let Some(ref stss) = &stbl.stss {
            boxes.push(build_box(stss, 5));
        }
        boxes.push(build_box(&stbl.stsc, 5));
        boxes.push(build_box(&stbl.stsz, 5));
        if let Some(ref stco) = &stbl.stco {
            boxes.push(build_box(stco, 5));
        }
        if let Some(ref co64) = &stbl.co64 {
            boxes.push(build_box(co64, 5));
        }
    }

    // If fragmented, add moof boxes.
    for moof in mp4.moofs.iter() {
        boxes.push(build_box(moof, 0));
        boxes.push(build_box(&moof.mfhd, 1));
        for traf in moof.trafs.iter() {
            boxes.push(build_box(traf, 2));
            boxes.push(build_box(&traf.tfhd, 3));
            if let Some(ref trun) = &traf.trun {
                boxes.push(build_box(trun, 3));
            }
        }
    }

    Ok(boxes)
}

fn descr_box(name: &str, indent: u32) -> Box {
    return Box {
        name: name.to_string(),
        size: 0,
        summary: "".to_string(),
        indent,
    };
}

fn build_box<M: Mp4Box + std::fmt::Debug>(ref m: &M, indent: u32) -> Box {
    return Box {
        name: m.box_type().to_string(),
        size: m.box_size(),
        summary: m.summary().unwrap(),
        indent: indent,
    };
}
