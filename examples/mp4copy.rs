use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader, BufWriter};
use std::path::Path;

use mp4::{
    AacConfig, AvcConfig, HevcConfig, MediaConfig, MediaType, Mp4Config, Result, TrackConfig,
    TtxtConfig, Vp9Config,
};

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

fn copy<P: AsRef<Path>>(src_filename: &P, dst_filename: &P) -> Result<()> {
    let src_file = File::open(src_filename)?;
    let size = src_file.metadata()?.len();
    let reader = BufReader::new(src_file);

    let dst_file = File::create(dst_filename)?;
    let writer = BufWriter::new(dst_file);

    let mut mp4_reader = mp4::Mp4Reader::read_header(reader, size)?;
    let mut mp4_writer = mp4::Mp4Writer::write_start(
        writer,
        &Mp4Config {
            major_brand: *mp4_reader.major_brand(),
            minor_version: mp4_reader.minor_version(),
            compatible_brands: mp4_reader.compatible_brands().to_vec(),
            timescale: mp4_reader.timescale(),
        },
    )?;

    // TODO interleaving
    for track in mp4_reader.tracks().values() {
        let media_conf = match track.media_type()? {
            MediaType::H264 => MediaConfig::AvcConfig(AvcConfig {
                width: track.width(),
                height: track.height(),
                seq_param_set: track.sequence_parameter_set()?.to_vec(),
                pic_param_set: track.picture_parameter_set()?.to_vec(),
            }),
            MediaType::H265 => MediaConfig::HevcConfig(HevcConfig {
                width: track.width(),
                height: track.height(),
            }),
            MediaType::VP9 => MediaConfig::Vp9Config(Vp9Config {
                width: track.width(),
                height: track.height(),
            }),
            MediaType::AAC => MediaConfig::AacConfig(AacConfig {
                bitrate: track.bitrate(),
                profile: track.audio_profile()?,
                freq_index: track.sample_freq_index()?,
                chan_conf: track.channel_config()?,
            }),
            MediaType::TTXT => MediaConfig::TtxtConfig(TtxtConfig {}),
        };

        let track_conf = TrackConfig {
            track_type: track.track_type()?,
            timescale: track.timescale(),
            language: track.language().to_string(),
            media_conf,
        };

        mp4_writer.add_track(&track_conf)?;
    }

    for track_id in mp4_reader.tracks().keys().copied().collect::<Vec<u32>>() {
        let sample_count = mp4_reader.sample_count(track_id)?;
        for sample_idx in 0..sample_count {
            let sample_id = sample_idx + 1;
            let sample = mp4_reader.read_sample(track_id, sample_id)?.unwrap();
            mp4_writer.write_sample(track_id, &sample)?;
            // println!("copy {}:({})", sample_id, sample);
        }
    }

    mp4_writer.write_end()?;

    Ok(())
}
