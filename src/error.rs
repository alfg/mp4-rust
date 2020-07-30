use thiserror::Error;

use crate::atoms::BoxType;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    #[error("{0}")]
    InvalidData(&'static str),
    #[error("{0} not found")]
    BoxNotFound(BoxType),
    #[error("trak[{0}] not found")]
    TrakNotFound(u32),
    #[error("trak[{0}].{1} not found")]
    BoxInTrakNotFound(u32, BoxType),
    #[error("trak[{0}].stbl.{1} not found")]
    BoxInStblNotFound(u32, BoxType),
    #[error("trak[{0}].stbl.{1}.entry[{2}] not found")]
    EntryInStblNotFound(u32, BoxType, u32),
}
