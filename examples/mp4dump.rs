use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::path::Path;

use mp4::{Result};

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
    let size = f.metadata()?.len();
    let reader = BufReader::new(f);

    let mp4 = mp4::Mp4Reader::read_header(reader, size)?;

    // ftyp
    println!("[{}] size={} ", mp4.ftyp.get_type(), mp4.ftyp.get_size());

    // moov
    println!("[{}] size={} ", mp4.moov.get_type(), mp4.moov.get_size());
    println!("  [{}] size={} ", mp4.moov.mvhd.get_type(), mp4.moov.mvhd.get_size());

    // Tracks.
    for track in mp4.tracks().iter() {

        // trak
        println!("  [{}] size={} ", track.trak.get_type(), track.trak.get_size());
        println!("    [{}] size={} ", track.trak.tkhd.get_type(), track.trak.tkhd.get_size());
        if let Some(ref edts) = track.trak.edts {
            println!("    [{}] size={} ", edts.get_type(), edts.get_size());
            if let Some(ref elst) = edts.elst {
                println!("      [{}] size={} ", elst.get_type(), elst.get_size());
            }
        }

        // trak.mdia.
        println!("    [{}] size={} ", track.trak.mdia.get_type(), track.trak.mdia.get_size());
        println!("      [{}] size={} ", track.trak.mdia.mdhd.get_type(), track.trak.mdia.mdhd.get_size());
        println!("      [{}] size={} ", track.trak.mdia.hdlr.get_type(), track.trak.mdia.hdlr.get_size());
        println!("      [{}] size={} ", track.trak.mdia.minf.get_type(), track.trak.mdia.minf.get_size());

        // trak.mdia.minf
        if let Some(ref vmhd) = track.trak.mdia.minf.vmhd {
            println!("        [{}] size={} ", vmhd.get_type(), vmhd.get_size());
        }
        if let Some(ref smhd) = track.trak.mdia.minf.smhd {
            println!("        [{}] size={} ", smhd.get_type(), smhd.get_size());
        }

        // trak.mdia.minf.stbl
        println!("      [{}] size={} ", track.trak.mdia.minf.stbl.get_type(), track.trak.mdia.minf.stbl.get_size());
        println!("        [{}] size={} ", track.trak.mdia.minf.stbl.stsd.get_type(), track.trak.mdia.minf.stbl.stsd.get_size());
        if let Some(ref avc1) = track.trak.mdia.minf.stbl.stsd.avc1 {
            println!("          [{}] size={} ", avc1.get_type(), avc1.get_size());
        }
        if let Some(ref mp4a) = track.trak.mdia.minf.stbl.stsd.mp4a {
            println!("          [{}] size={} ", mp4a.get_type(), mp4a.get_size());
        }
        println!("        [{}] size={} ", track.trak.mdia.minf.stbl.stts.get_type(), track.trak.mdia.minf.stbl.stts.get_size());
        if let Some(ref ctts) = track.trak.mdia.minf.stbl.ctts {
            println!("        [{}] size={} ", ctts.get_type(), ctts.get_size());
        }
        if let Some(ref stss) = track.trak.mdia.minf.stbl.stss {
            println!("        [{}] size={} ", stss.get_type(), stss.get_size());
        }
        println!("        [{}] size={} ", track.trak.mdia.minf.stbl.stsc.get_type(), track.trak.mdia.minf.stbl.stsc.get_size());
        println!("        [{}] size={} ", track.trak.mdia.minf.stbl.stsz.get_type(), track.trak.mdia.minf.stbl.stsz.get_size());
        if let Some(ref stco) = track.trak.mdia.minf.stbl.stco {
            println!("        [{}] size={} ", stco.get_type(), stco.get_size());
        }
        if let Some(ref co64) = track.trak.mdia.minf.stbl.co64 {
            println!("        [{}] size={} ", co64.get_type(), co64.get_size());
        }
    }

    Ok(())
}