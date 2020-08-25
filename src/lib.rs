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