use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AdrmBox {
    #[serde(serialize_with = "<[_]>::serialize")]
    pub drm_blob: [u8; 48],
    pub file_checksum: [u8; 20],
    #[serde(serialize_with = "<[_]>::serialize")]
    pub unknown0: [u8; 60],
}

impl Mp4Box for AdrmBox {
    fn box_type(&self) -> BoxType {
        BoxType::AdrmBox
    }

    fn box_size(&self) -> u64 {
        156
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let mut s = String::with_capacity(49);
        s.push_str("checksum=");
        for b in self.file_checksum.iter() {
            s.push_str(&format!("{:02x}", b));
        }
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for AdrmBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut result = AdrmBox {
            drm_blob: [0u8; 48],
            file_checksum: [0u8; 20],
            unknown0: [0u8; 60],
        };

        reader.read_u32::<BigEndian>()?; // 56
        reader.read_u32::<BigEndian>()?; // 1
        reader.read_exact(&mut result.drm_blob)?;
        reader.read_u32::<BigEndian>()?; // 0
        reader.read_u32::<BigEndian>()?; // 1
        reader.read_u32::<BigEndian>()?; // 0
        reader.read_exact(&mut result.file_checksum)?;
        reader.read_exact(&mut result.unknown0)?;

        assert_eq!(reader.stream_position()?, start + size);

        Ok(result)
    }
}

impl<W: Write> WriteBox<&mut W> for AdrmBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        writer.write_u32::<BigEndian>(56)?;
        writer.write_u32::<BigEndian>(1)?;
        writer.write_all(&self.drm_blob)?;
        writer.write_u32::<BigEndian>(0)?;
        writer.write_u32::<BigEndian>(1)?;
        writer.write_u32::<BigEndian>(0)?;
        writer.write_all(&self.file_checksum)?;
        writer.write_all(&self.unknown0)?;

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mp4box::BoxHeader;
    use std::io::Cursor;

    #[test]
    fn test_adrm() {
        let src_box = AdrmBox {
            drm_blob: [29u8; 48],
            file_checksum: [244u8; 20],
            unknown0: [113u8; 60],
        };

        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::AdrmBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = AdrmBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
