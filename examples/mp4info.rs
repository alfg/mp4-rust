use std::env;
use std::io::prelude::*;
use std::io;
use std::path::Path;

#[cfg(not(feature = "async"))]
use {
    std::fs::File,
    std::io::BufReader,
};

#[cfg(feature = "async")]
use {
    tokio::fs::File,
};

use mp4::{Mp4Track, Result, TrackType};

#[cfg(not(feature = "async"))]
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


#[cfg(feature = "async")]
#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: mp4info <filename>");
        std::process::exit(1);
    }

    if let Err(err) = async_info(&args[1]).await {
        let _ = writeln!(io::stderr(), "{}", err);
    }
}

#[cfg(not(feature = "async"))]
fn info<P: AsRef<Path>>(filename: &P) -> Result<()> {
    let f = File::open(filename)?;
    let size = f.metadata()?.len();
    let reader = BufReader::new(f);

    let mp4 = mp4::Mp4Reader::read_header(reader, size)?;

    println!("Metadata:");
    println!("  size            : {}", mp4.size());
    println!("  major_brand     : {}", mp4.major_brand());
    println!("  minor_version   : {}", mp4.minor_version());
    let mut compatible_brands = String::new();
    for brand in mp4.compatible_brands().iter() {
        compatible_brands.push_str(&brand.to_string());
        compatible_brands.push_str(",");
    }
    println!("  compatible_brands: {}", compatible_brands);
    println!("Duration: {:?}, timescale: {}", mp4.duration(), mp4.timescale());

    for track in mp4.tracks().iter() {
        let media_info = match track.track_type()? {
            TrackType::Video => video_info(track)?,
            TrackType::Audio => audio_info(track)?,
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

#[cfg(feature = "async")]
async fn async_info<P: AsRef<Path>>(filename: &P) -> Result<()> {
    let file = File::open(filename).await?;
    let size = file.metadata().await?.len();

    let mp4 = mp4::Mp4AsyncReader::async_read_header(file, size).await?;

    println!("Metadata:");
    println!("  size            : {}", mp4.size());
    println!("  major_brand     : {}", mp4.major_brand());
    println!("  minor_version   : {}", mp4.minor_version());
    let mut compatible_brands = String::new();
    for brand in mp4.compatible_brands().iter() {
        compatible_brands.push_str(&brand.to_string());
        compatible_brands.push_str(",");
    }
    println!("  compatible_brands: {}", compatible_brands);
    println!("Duration: {:?}", mp4.duration());

    for track in mp4.tracks().iter() {
        let media_info = match track.track_type()? {
            TrackType::Video => video_info(track)?,
            TrackType::Audio => audio_info(track)?,
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
    Ok(format!(
        "{} ({}) ({:?}), {}, {}x{}, {} kb/s, {:.2} fps",
        track.media_type()?,
        track.video_profile()?,
        track.box_type()?,
        track.timescale(),
        track.width(),
        track.height(),
        track.bitrate() / 1000,
        track.frame_rate_f64()
    ))
}

fn audio_info(track: &Mp4Track) -> Result<String> {
    Ok(format!(
        "{} ({}) ({:?}), {}, {} Hz, {}, {} kb/s",
        track.media_type()?,
        track.audio_profile()?,
        track.box_type()?,
        track.timescale(),
        track.sample_freq_index()?.freq(),
        track.channel_config()?,
        track.bitrate() / 1000
    ))
}
