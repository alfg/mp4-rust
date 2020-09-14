use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, Write};
use serde::{Serialize};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct TrunBox {
    pub version: u8,
    pub flags: u32,
    pub sample_count: u32,
    pub data_offset: i32,

    #[serde(skip_serializing)]
    pub sample_sizes: Vec<u32>,
}

impl TrunBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::TrunBox
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 8 + (4 * self.sample_sizes.len() as u64)
    }
}

impl Mp4Box for TrunBox {
    fn box_type(&self) -> BoxType {
        return self.get_type();
    }

    fn box_size(&self) -> u64 {
        return self.get_size();
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("sample_size={}",
            self.sample_count);
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for TrunBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        let sample_count = reader.read_u32::<BigEndian>()?;
        let data_offset = reader.read_i32::<BigEndian>()?;

        let mut sample_sizes = Vec::with_capacity(sample_count as usize);
        for _ in 0..sample_count {
            let sample_size = reader.read_u32::<BigEndian>()?;
            sample_sizes.push(sample_size);
        }

        skip_bytes_to(reader, start + size)?;

        Ok(TrunBox {
            version,
            flags,
            sample_count,
            data_offset,
            sample_sizes,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for TrunBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        writer.write_u32::<BigEndian>(self.sample_count)?;
        writer.write_i32::<BigEndian>(self.data_offset)?;
        assert_eq!(self.sample_count, self.sample_sizes.len() as u32);
        for sample_number in self.sample_sizes.iter() {
            writer.write_u32::<BigEndian>(*sample_number)?;
        }

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mp4box::BoxHeader;
    use std::io::Cursor;

    #[test]
    fn test_trun_same_size() {
        let src_box = TrunBox {
            version: 0,
            flags: 0,
            data_offset: 0,
            sample_count: 0,
            sample_sizes: vec![],
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::TrunBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = TrunBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }

    #[test]
    fn test_trun_many_sizes() {
        let src_box = TrunBox {
            version: 0,
            flags: 0,
            data_offset: 0,
            sample_count: 9,
            sample_sizes: vec![1165, 11, 11, 8545, 10126, 10866, 9643, 9351, 7730],
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::TrunBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = TrunBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
