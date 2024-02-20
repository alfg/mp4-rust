use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct SaizBox {
    pub aux_info: Option<AuxiliaryInfoType>,
    pub default_sample_info_size: u8,
    pub sample_count: u32,
    pub sample_info_sizes: Vec<u8>,
}

impl SaizBox {
    pub const FLAG_AUX_INFO_TYPE: u32 = 0x01;

    pub fn get_type(&self) -> BoxType {
        BoxType::SaizBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE + 5;
        if self.aux_info.is_some() {
            size += 8;
        }
        if self.default_sample_info_size == 0 {
            size += self.sample_count as u64;
        }
        size
    }
}

impl Mp4Box for SaizBox {
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
            "sample_info_size={} sample_count={}",
            self.default_sample_info_size, self.sample_count
        );
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for SaizBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (_version, flags) = read_box_header_ext(reader)?;

        let mut aux_info = None;
        if SaizBox::FLAG_AUX_INFO_TYPE & flags != 0 {
            let aux_info_type = reader.read_u32::<BigEndian>()?;
            let aux_info_type_parameter = reader.read_u32::<BigEndian>()?;
            aux_info = Some(AuxiliaryInfoType {
                aux_info_type,
                aux_info_type_parameter,
            });
        }

        let default_sample_info_size = reader.read_u8()?;
        let sample_count = reader.read_u32::<BigEndian>()?;

        let mut sample_info_sizes = Vec::new();
        if default_sample_info_size == 0 {
            sample_info_sizes = Vec::with_capacity(sample_count as usize);
            for _ in 0..sample_count as usize {
                sample_info_sizes.push(reader.read_u8()?);
            }
        };

        skip_bytes_to(reader, start + size)?;

        Ok(SaizBox {
            aux_info,
            default_sample_info_size,
            sample_count,
            sample_info_sizes,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for SaizBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        if let Some(ref aux_info) = self.aux_info {
            write_box_header_ext(writer, 0, SaizBox::FLAG_AUX_INFO_TYPE)?;
            writer.write_u32::<BigEndian>(aux_info.aux_info_type)?;
            writer.write_u32::<BigEndian>(aux_info.aux_info_type_parameter)?;
        } else {
            write_box_header_ext(writer, 0, 0)?;
        }

        writer.write_u8(self.default_sample_info_size)?;
        writer.write_u32::<BigEndian>(self.sample_count)?;

        if self.default_sample_info_size == 0 {
            for i in 0..self.sample_count as usize {
                let sample_info_size = self.sample_info_sizes.get(i).copied().unwrap_or(0);
                writer.write_u8(sample_info_size)?;
            }
        }
        Ok(size)
    }
}
