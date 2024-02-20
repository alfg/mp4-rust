use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct SaioBox {
    pub version: u8,
    pub aux_info: Option<AuxiliaryInfoType>,
    pub entry_count: u32,
    pub offsets: Vec<u64>,
}

impl SaioBox {
    pub const FLAG_AUX_INFO_TYPE: u32 = 0x01;

    pub fn get_type(&self) -> BoxType {
        BoxType::SaioBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE + 4;
        if self.aux_info.is_some() {
            size += 8;
        }
        if self.version == 0 {
            size += 4 * self.entry_count as u64;
        } else {
            size += 8 * self.entry_count as u64;
        }
        size
    }
}

impl Mp4Box for SaioBox {
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
        let s = format!("entry_count={}", self.entry_count);
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for SaioBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        let mut aux_info = None;
        if SaioBox::FLAG_AUX_INFO_TYPE & flags != 0 {
            let aux_info_type = reader.read_u32::<BigEndian>()?;
            let aux_info_type_parameter = reader.read_u32::<BigEndian>()?;
            aux_info = Some(AuxiliaryInfoType {
                aux_info_type,
                aux_info_type_parameter,
            });
        }

        let sample_count = reader.read_u32::<BigEndian>()?;

        let mut offsets = Vec::with_capacity(sample_count as usize);
        for _ in 0..sample_count as usize {
            let offset = if version == 0 {
                reader.read_u32::<BigEndian>()? as u64
            } else {
                reader.read_u64::<BigEndian>()?
            };
            offsets.push(offset);
        }

        skip_bytes_to(reader, start + size)?;

        Ok(SaioBox {
            version,
            aux_info,
            entry_count: sample_count,
            offsets,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for SaioBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        if let Some(ref aux_info) = self.aux_info {
            write_box_header_ext(writer, self.version, SaioBox::FLAG_AUX_INFO_TYPE)?;
            writer.write_u32::<BigEndian>(aux_info.aux_info_type)?;
            writer.write_u32::<BigEndian>(aux_info.aux_info_type_parameter)?;
        } else {
            write_box_header_ext(writer, self.version, 0)?;
        }

        writer.write_u32::<BigEndian>(self.entry_count)?;

        for i in 0..self.entry_count as usize {
            let offset = self.offsets.get(i).copied().unwrap_or(0);
            if self.version == 0 {
                writer.write_u32::<BigEndian>(offset as u32)?;
            } else {
                writer.write_u64::<BigEndian>(offset)?;
            };
        }

        Ok(size)
    }
}
