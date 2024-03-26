use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct UuidBox {
    pub extended_type: [u8; 16],
    pub data: Vec<u8>,
}

impl UuidBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::UuidBox
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE + 16 + self.data.len() as u64
    }
}

impl Mp4Box for UuidBox {
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
        let s = format!("extended_type: {:02x?}", self.extended_type);
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for UuidBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut extended_type = [0; 16];
        reader.read_exact(&mut extended_type)?;

        let data_size = (start + size)
            .checked_sub(reader.stream_position()?)
            .ok_or(Error::InvalidData("uuid size too small"))?;
        let mut data = vec![0; data_size as usize];
        reader.read_exact(&mut data)?;

        skip_bytes_to(reader, start + size)?;

        Ok(UuidBox {
            extended_type,
            data,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for UuidBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();

        BoxHeader::new(self.box_type(), size).write(writer)?;

        writer.write_all(&self.extended_type)?;
        writer.write_all(&self.data)?;

        Ok(size)
    }
}
