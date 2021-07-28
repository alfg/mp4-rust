use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::path::Path;

use mp4::{Mp4Track, Result, TrackType, Error};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: mp4info <filename>");
        std::process::exit(1);
    }

    if let Err(err) = info(&args[1]) {
        let _ = writeln!(io::stderr(), "{}", err);
    }
}

fn info<P: AsRef<Path>>(filename: &P) -> Result<()> {
    let f = File::open(filename)?;
    let size = f.metadata()?.len();
    let reader = BufReader::new(f);

    let mp4 = mp4::Mp4Reader::read_header(reader, size)?;

    println!("File:");
    println!("  file size:          {}", mp4.size());
    println!("  major_brand:        {}", mp4.major_brand());
    let mut compatible_brands = String::new();
    for brand in mp4.compatible_brands().iter() {
        compatible_brands.push_str(&brand.to_string());
        compatible_brands.push_str(" ");
    }
    println!("  compatible_brands:  {}\n", compatible_brands);

    println!("Movie:");
    println!("  version:        {}", mp4.moov.mvhd.version);
    println!("  creation time:  {}", creation_time(mp4.moov.mvhd.creation_time));
    println!("  duration:       {:?}", mp4.duration());
    println!("  fragments:      {:?}", mp4.is_fragmented());
    println!("  timescale:      {:?}\n", mp4.timescale());

    println!("Found {} Tracks", mp4.tracks().len());
    for track in mp4.tracks().values() {
        let media_info = match track.track_type()? {
            TrackType::Video => video_info(track)?,
            TrackType::Audio => audio_info(track)?,
            TrackType::Subtitle => subtitle_info(track)?,
        };
        println!(
            "  Track: #{}({}) {}: {}",
            track.track_id(),
            track.language(),
            track.track_type()?,
            media_info
        );
    }
    Ok(())
}

fn video_info(track: &Mp4Track) -> Result<String> {
    if track.trak.mdia.minf.stbl.stsd.avc1.is_some() {
        Ok(format!(
            "{} ({}) ({:?}), {}x{}, {} kb/s, {:.2} fps",
            track.media_type()?,
            track.video_profile()?,
            track.box_type()?,
            track.width(),
            track.height(),
            track.bitrate() / 1000,
            track.frame_rate()
        ))
    } else {
        Ok(format!(
            "{} ({:?}), {}x{}, {} kb/s, {:.2} fps",
            track.media_type()?,
            track.box_type()?,
            track.width(),
            track.height(),
            track.bitrate() / 1000,
            track.frame_rate()
        ))
    }
}

fn audio_info(track: &Mp4Track) -> Result<String> {
    if let Some(ref mp4a) = track.trak.mdia.minf.stbl.stsd.mp4a {
        if mp4a.esds.is_some() {

            let profile = match track.audio_profile() {
                Ok(val) => val.to_string(),
                _ => "-".to_string(),
            };

            let channel_config = match track.channel_config() {
                Ok(val) => val.to_string(),
                _ => "-".to_string(),
            };

            Ok(format!(
                "{} ({}) ({:?}), {} Hz, {}, {} kb/s",
                track.media_type()?,
                profile,
                track.box_type()?,
                track.sample_freq_index()?.freq(),
                channel_config,
                track.bitrate() / 1000
            ))
        } else {
            Ok(format!(
                "{} ({:?}), {} kb/s",
                track.media_type()?,
                track.box_type()?,
                track.bitrate() / 1000
            ))
        }
    } else {
        Err(Error::InvalidData("mp4a box not found"))
    }
}

fn subtitle_info(track: &Mp4Track) -> Result<String> {
    if track.trak.mdia.minf.stbl.stsd.tx3g.is_some() {
        Ok(format!(
            "{} ({:?})",
            track.media_type()?,
            track.box_type()?,
        ))
    } else {
        Err(Error::InvalidData("tx3g box not found"))
    }
}

fn creation_time(creation_time: u64) -> u64 {
    // convert from MP4 epoch (1904-01-01) to Unix epoch (1970-01-01)
    if creation_time >= 2082844800 {
        creation_time - 2082844800
    } else {
        creation_time
    }
}