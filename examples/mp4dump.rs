use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::path::Path;

use mp4::{Result, Mp4Box};

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
       println!("[{}] size={}", b.name, b.size); 
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Box {
    name: String,
    size: u64,
    indent: u32,
}

fn get_boxes(file: File) -> Result<Vec<Box>> {
    let size = file.metadata()?.len();
    let reader = BufReader::new(file);
    let mp4 = mp4::Mp4Reader::read_header(reader, size)?;

    // collect known boxes
    let mut boxes = Vec::new();

    // ftyp, moov, mvhd
    boxes.push(build_box(&mp4.ftyp));
    boxes.push(build_box(&mp4.moov));
    boxes.push(build_box(&mp4.moov.mvhd));

    // trak.
    for track in mp4.tracks().iter() {
        boxes.push(build_box(&track.trak));
        boxes.push(build_box(&track.trak.tkhd));
        if let Some(ref edts) = track.trak.edts {
            boxes.push(build_box(edts));
            if let Some(ref elst) = edts.elst {
                boxes.push(build_box(elst));
            }
        }

        // trak.mdia
        let mdia = &track.trak.mdia;
        boxes.push(build_box(mdia));
        boxes.push(build_box(&mdia.mdhd));
        boxes.push(build_box(&mdia.hdlr));
        boxes.push(build_box(&track.trak.mdia.minf));

        // trak.mdia.minf
        let minf = &track.trak.mdia.minf;
        if let Some(ref vmhd) = &minf.vmhd {
            boxes.push(build_box(vmhd));
        }
        if let Some(ref smhd) = &minf.smhd {
            boxes.push(build_box(smhd));
        }

        // trak.mdia.minf.stbl
        let stbl = &track.trak.mdia.minf.stbl;
        boxes.push(build_box(stbl));
        boxes.push(build_box(&stbl.stsd));
        if let Some(ref avc1) = &stbl.stsd.avc1 {
            boxes.push(build_box(avc1));
        }
        if let Some(ref hev1) = &stbl.stsd.hev1 {
            boxes.push(build_box(hev1));
        }
        if let Some(ref mp4a) = &stbl.stsd.mp4a {
            boxes.push(build_box(mp4a));
        }
        boxes.push(build_box(&stbl.stts));
        if let Some(ref ctts) = &stbl.ctts {
            boxes.push(build_box(ctts));
        }
        if let Some(ref stss) = &stbl.stss {
            boxes.push(build_box(stss));
        }
        boxes.push(build_box(&stbl.stsc));
        boxes.push(build_box(&stbl.stsz));
        if let Some(ref stco) = &stbl.stco {
            boxes.push(build_box(stco));
        }
        if let Some(ref co64) = &stbl.co64 {
            boxes.push(build_box(co64));
        }
    }

    // If fragmented, add moof boxes.
    for moof in mp4.moofs.iter() {
        boxes.push(build_box(moof));
        boxes.push(build_box(&moof.mfhd));
        for traf in moof.trafs.iter() {
            boxes.push(build_box(traf));
            boxes.push(build_box(&traf.tfhd));
        }
    }

    Ok(boxes)
}

fn build_box<M: Mp4Box + std::fmt::Debug>(ref m: &M) -> Box {
    return Box{
        name: m.box_type().to_string(),
        size: m.box_size(),
        indent: 0,
    };
}
