use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

const GPS_DATA_BLOCK_HEADER_SIZE: u64 = 8;
const GPS_DATA_BLOCK_INFO_SIZE: u64 = 8;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize)]
pub struct GpsDataBlockInfo {
    /// File offset of GPS data block in bytes
    pub offset: u32,
    /// Size of GPS data block in bytes
    pub size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct GpsBox {
    pub version_and_date: u64,
    pub data_blocks: Vec<GpsDataBlockInfo>,
}

impl GpsBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::GpsBox
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE
            + GPS_DATA_BLOCK_HEADER_SIZE
            + (self.data_blocks.len() as u64 * GPS_DATA_BLOCK_INFO_SIZE)
    }
}

impl Mp4Box for GpsBox {
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
            "version_and_date=0x{:X}, num_blocks={}",
            self.version_and_date,
            self.data_blocks.len()
        );
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for GpsBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let version_and_date = reader.read_u64::<BigEndian>()?;

        // TODO - size checks/etc
        let mut data_blocks = Vec::new();
        let count = (size - HEADER_SIZE - GPS_DATA_BLOCK_HEADER_SIZE) / GPS_DATA_BLOCK_INFO_SIZE;
        for _ in 0..count {
            let offset = reader.read_u32::<BigEndian>()?;
            let size = reader.read_u32::<BigEndian>()?;
            if offset == 0 || size == 0 {
                // log::warn!("Ignoring block offset={}, size={}", offset, size);
            } else {
                data_blocks.push(GpsDataBlockInfo { offset, size });
            }
        }

        skip_bytes_to(reader, start + size)?;

        Ok(GpsBox {
            version_and_date,
            data_blocks,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for GpsBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;
        writer.write_u64::<BigEndian>(self.version_and_date)?;
        for info in self.data_blocks.iter() {
            writer.write_u32::<BigEndian>(info.offset)?;
            writer.write_u32::<BigEndian>(info.size)?;
        }
        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    // TODO
}
