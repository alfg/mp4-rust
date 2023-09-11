use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct SchmBox {
    pub version: u8,
    pub scheme_type: FourCC,
    pub scheme_version: u32,
}

impl SchmBox {
    pub const FLAG_SCHEME_URI: u32 = 0x01;

    pub fn get_type(&self) -> BoxType {
        BoxType::SchmBox
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 8
    }
}

impl Mp4Box for SchmBox {
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
        let s = format!(
            "scheme_type={} scheme_version={}",
            self.scheme_type, self.scheme_version
        );
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for SchmBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        let scheme_type = reader.read_u32::<BigEndian>()?;
        let scheme_version = reader.read_u32::<BigEndian>()?;

        if SchmBox::FLAG_SCHEME_URI & flags != 0 {
            // todo
        }

        skip_bytes_to(reader, start + size)?;

        Ok(SchmBox {
            version,
            scheme_type: scheme_type.into(),
            scheme_version,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for SchmBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, 0)?;

        writer.write_u32::<BigEndian>(self.scheme_type.into())?;
        writer.write_u32::<BigEndian>(self.scheme_version)?;

        Ok(size)
    }
}
