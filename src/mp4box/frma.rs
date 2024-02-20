use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct FrmaBox {
    pub original_format: FourCC,
}

impl FrmaBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::FrmaBox
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE + 4
    }
}

impl Mp4Box for FrmaBox {
    fn box_type(&self) -> BoxType {
        self.get_type()
    }

    fn box_size(&self) -> u64 {
        self.get_size()
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("original_format={}", self.original_format,);
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for FrmaBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let original_format = reader.read_u32::<BigEndian>()?;

        skip_bytes_to(reader, start + size)?;

        Ok(FrmaBox {
            original_format: original_format.into(),
        })
    }
}

impl<W: Write> WriteBox<&mut W> for FrmaBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        writer.write_u32::<BigEndian>(self.original_format.into())?;

        Ok(size)
    }
}
