mod error;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

mod types;
pub use types::*;

mod mp4box;

mod track;
pub use track::{Mp4Track, TrackConfig};

mod reader;
pub use reader::Mp4Reader;

mod writer;
pub use writer::{Mp4Config, Mp4Writer};
