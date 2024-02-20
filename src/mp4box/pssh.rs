use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct PsshBox {
    pub version: u8,
    pub system_id: [u8; 16],
    pub kids: Vec<[u8; 16]>,
    pub data: Vec<u8>,
}

impl PsshBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::PsshBox
    }

    pub fn get_size(&self) -> u64 {
        let mut s = HEADER_SIZE + HEADER_EXT_SIZE + 16;
        if self.version > 0 {
            s += 4 + (16 * self.kids.len() as u64);
        }
        s += 4 + self.data.len() as u64;
        s
    }
}

impl Mp4Box for PsshBox {
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
            "system_id={:02x?} data_size={}",
            self.system_id,
            self.data.len(),
        );
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for PsshBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, _flags) = read_box_header_ext(reader)?;

        let mut system_id = [0; 16];
        reader.read_exact(&mut system_id)?;

        let mut kids = Vec::new();
        if version > 0 {
            let kid_count = reader.read_u32::<BigEndian>()?;
            kids.reserve(kid_count as usize);
            for _ in 0..kid_count {
                let mut kid = [0; 16];
                reader.read_exact(&mut kid)?;
                kids.push(kid);
            }
        }

        let data_size = reader.read_u32::<BigEndian>()?;

        let mut data = vec![0; data_size as usize];
        reader.read_exact(&mut data)?;

        skip_bytes_to(reader, start + size)?;

        Ok(PsshBox {
            version,
            system_id,
            kids,
            data,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for PsshBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, 0)?;

        writer.write_all(&self.system_id)?;

        if self.version > 0 {
            writer.write_u32::<BigEndian>(self.kids.len() as u32)?;
            for kid in &self.kids {
                writer.write_all(kid)?;
            }
        }

        writer.write_u32::<BigEndian>(self.data.len() as u32)?;
        writer.write_all(&self.data)?;

        Ok(size)
    }
}
