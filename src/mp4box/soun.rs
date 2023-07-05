use serde::Serialize;

use crate::mp4box::*;

// for opus
// https://opus-codec.org/docs/opus_in_isobmff.html
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SounBox {}

impl Default for SounBox {
    fn default() -> Self {
        Self {}
    }
}

impl Mp4Box for SounBox {
    fn box_type(&self) -> BoxType {
        BoxType::SounBox
    }

    fn box_size(&self) -> u64 {
        todo!()
    }

    fn to_json(&self) -> Result<String> {
        serde_json::to_string(&self).map_err(|e| crate::error::Error::IoError(e.into()))
    }

    fn summary(&self) -> Result<String> {
        todo!()
    }
}
