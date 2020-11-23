//! `mp4` is a Rust library to read and write ISO-MP4 files.
//! 
//! This package contains MPEG-4 specifications defined in parts:
//!    * ISO/IEC 14496-12 - ISO Base Media File Format (QuickTime, MPEG-4, etc)
//!    * ISO/IEC 14496-14 - MP4 file format
//!    * ISO/IEC 14496-17 - Streaming text format
//! 
//! See: [mp4box] for supported MP4 atoms.
//! 
//! ### Example
//! 
//! ```
//! use std::fs::File;
//! use std::io::{BufReader};
//! use mp4::{Result};
//!
//! fn main() -> Result<()> {
//!     let f = File::open("tests/samples/minimal.mp4").unwrap();
//!     let size = f.metadata()?.len();
//!     let reader = BufReader::new(f);
//!
//!     let mp4 = mp4::Mp4Reader::read_header(reader, size)?;
//!
//!     // Print boxes.
//!     println!("major brand: {}", mp4.ftyp.major_brand);
//!     println!("timescale: {}", mp4.moov.mvhd.timescale);
//!
//!     // Use available methods.
//!     println!("size: {}", mp4.size());
//!
//!     let mut compatible_brands = String::new();
//!     for brand in mp4.compatible_brands().iter() {
//!         compatible_brands.push_str(&brand.to_string());
//!         compatible_brands.push_str(",");
//!     }
//!     println!("compatible brands: {}", compatible_brands);
//!     println!("duration: {:?}", mp4.duration());
//!
//!    // Track info.
//!    for track in mp4.tracks().iter() {
//!        println!(
//!            "track: #{}({}) {} : {}",
//!            track.track_id(),
//!            track.language(),
//!            track.track_type()?,
//!            track.box_type()?,
//!        );
//!    }
//!    Ok(())
//! }
//! ```
//! 
//! See [examples] for more examples.
//! 
//! # Installation
//!
//! Add the following to your `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! mp4 = "0.7.0"
//! ```
//! 
//! [mp4box]: https://github.com/alfg/mp4-rust/blob/master/src/mp4box/mod.rs
//! [examples]: https://github.com/alfg/mp4-rust/blob/master/src/examples
#![doc(html_root_url = "https://docs.rs/mp4/*")]


use std::io::{BufReader};
use std::fs::File;

mod error;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

mod types;
pub use types::*;

mod mp4box;
pub use mp4box::{Mp4Box};

mod track;
pub use track::{Mp4Track, TrackConfig};

mod reader;
pub use reader::Mp4Reader;

mod writer;
pub use writer::{Mp4Config, Mp4Writer};

pub fn read_mp4(f: File) -> Result<Mp4Reader<BufReader<File>>> {
    let size = f.metadata()?.len();
    let reader = BufReader::new(f);
    let mp4 = reader::Mp4Reader::read_header(reader, size)?;
    Ok(mp4)
}