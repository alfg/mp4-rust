use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct TfhdBox {
    pub version: u8,
    pub flags: u32,
    pub track_id: u32,
    pub base_data_offset: u64,
}

impl TfhdBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::TfhdBox
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 4 + 8
    }
}

impl Mp4Box for TfhdBox {
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
        let s = format!("track_id={}", self.track_id);
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for TfhdBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;
        let track_id = reader.read_u32::<BigEndian>()?;
        let base_data_offset = reader.read_u64::<BigEndian>()?;

        skip_bytes_to(reader, start + size)?;

        Ok(TfhdBox {
            version,
            flags,
            track_id,
            base_data_offset,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for TfhdBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;
        writer.write_u32::<BigEndian>(self.track_id)?;
        writer.write_u64::<BigEndian>(self.base_data_offset)?;

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mp4box::BoxHeader;
    use std::io::Cursor;

    #[test]
    fn test_tfhd() {
        let src_box = TfhdBox {
            version: 0,
            flags: 0,
            track_id: 1,
            base_data_offset: 0,
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::TfhdBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = TfhdBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
