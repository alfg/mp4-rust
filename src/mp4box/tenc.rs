use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct TencBox {
    pub version: u8,
    pub default_crypt_byte_block: u8,
    pub default_skip_byte_block: u8,
    pub default_is_protected: bool,
    pub default_per_sample_iv_size: u8,
    pub default_kid: [u8; 16],
    pub default_constant_iv: Vec<u8>,
}

impl TencBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::TencBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE + 20;
        if self.default_is_protected && self.default_per_sample_iv_size == 0 {
            size += 1 + (self.default_constant_iv.len() & 0xff) as u64;
        }
        size
    }
}

impl Mp4Box for TencBox {
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
        let mut s = format!(
            "crypt_byte_block={} skip_byte_block={} protected={} iv_size={} kid={:x?}",
            self.default_crypt_byte_block,
            self.default_skip_byte_block,
            self.default_is_protected,
            self.default_per_sample_iv_size,
            self.default_kid,
        );
        if !self.default_constant_iv.is_empty() {
            s.push_str(&format!(" constant_iv={:x?}", self.default_constant_iv));
        }
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for TencBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut default_crypt_byte_block = 0;
        let mut default_skip_byte_block = 0;

        let (version, _flags) = read_box_header_ext(reader)?;

        let _reserved = reader.read_u8()?;
        let val = reader.read_u8()?;
        if version > 0 {
            default_crypt_byte_block = val >> 4;
            default_skip_byte_block = val & 0xf;
        }
        let default_is_protected = reader.read_u8()? != 0;
        let default_per_sample_iv_size = reader.read_u8()?;

        let mut default_kid = [0; 16];
        reader.read_exact(&mut default_kid)?;

        let mut default_constant_iv = Vec::new();
        if default_is_protected && default_per_sample_iv_size == 0 {
            let default_constant_iv_size = reader.read_u8()?;
            if default_constant_iv_size > 0 {
                default_constant_iv = vec![0; default_constant_iv_size as usize];
                reader.read_exact(&mut default_constant_iv[..])?;
            }
        }

        skip_bytes_to(reader, start + size)?;

        Ok(TencBox {
            version,
            default_crypt_byte_block,
            default_skip_byte_block,
            default_is_protected,
            default_per_sample_iv_size,
            default_kid,
            default_constant_iv,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for TencBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, 0)?;

        writer.write_u8(0)?; // reserved
        let val = if self.version > 0 {
            (self.default_crypt_byte_block << 4) | (self.default_skip_byte_block & 0xf)
        } else {
            0
        };
        writer.write_u8(val)?;
        writer.write_u8(self.default_is_protected as u8)?;
        writer.write_u8(self.default_per_sample_iv_size)?;
        writer.write_all(&self.default_kid)?;
        if self.default_is_protected && self.default_per_sample_iv_size == 0 {
            let default_constant_iv_size = (self.default_constant_iv.len() & 0xff) as u8;
            writer.write_u8(default_constant_iv_size)?;
            writer.write_all(&self.default_constant_iv[0..default_constant_iv_size as usize])?;
        }

        Ok(size)
    }
}
