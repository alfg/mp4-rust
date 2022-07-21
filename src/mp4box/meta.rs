use std::io::{Read, Seek};

use serde::Serialize;

use crate::mp4box::ilst::IlstBox;
use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct MetaBox {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ilst: Option<IlstBox>,
}

impl<R: Read + Seek> ReadBox<&mut R> for MetaBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, _) = read_box_header_ext(reader)?;
        if version != 0 {
            return Err(Error::UnsupportedBoxVersion(
                BoxType::UdtaBox,
                version as u8,
            ));
        }

        let mut ilst = None;

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader { name, size: s } = header;

            match name {
                BoxType::IlstBox => {
                    ilst = Some(IlstBox::read_box(reader, s)?);
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }

            current = reader.seek(SeekFrom::Current(0))?;
        }

        skip_bytes_to(reader, start + size)?;

        Ok(MetaBox { ilst })
    }
}
