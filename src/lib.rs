mod error;
pub use error::Error;

mod types;
pub use types::*;

mod atoms;

mod reader;
pub use reader::Mp4Reader;

mod track;
pub use track::Mp4Track;

pub type Result<T> = std::result::Result<T, Error>;
