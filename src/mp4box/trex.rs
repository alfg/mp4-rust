use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, Write};
use serde::{Serialize};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct TrexBox {
    pub version: u8,
    pub flags: u32,
    pub track_id: u32,
    pub default_sample_description_index: u32, 
    pub default_sample_duration: u32, 
    pub default_sample_size: u32, 
    pub default_sample_flags: u32, 
}

impl TrexBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::TrexBox
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 4 + 20
    }
}

impl Mp4Box for TrexBox {
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
        let s = format!("track_id={} default_sample_duration={}",
            self.track_id, self.default_sample_duration);
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for TrexBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        reader.read_u32::<BigEndian>()?; // pre-defined
        let track_id = reader.read_u32::<BigEndian>()?;
        let default_sample_description_index = reader.read_u32::<BigEndian>()?;
        let default_sample_duration = reader.read_u32::<BigEndian>()?;
        let default_sample_size = reader.read_u32::<BigEndian>()?;
        let default_sample_flags = reader.read_u32::<BigEndian>()?;

        skip_bytes_to(reader, start + size)?;

        Ok(TrexBox {
            version,
            flags,
            track_id,
            default_sample_description_index,
            default_sample_duration,
            default_sample_size,
            default_sample_flags,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for TrexBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        writer.write_u32::<BigEndian>(0)?; // pre-defined
        writer.write_u32::<BigEndian>(self.track_id)?;
        writer.write_u32::<BigEndian>(self.default_sample_description_index)?;
        writer.write_u32::<BigEndian>(self.default_sample_duration)?;
        writer.write_u32::<BigEndian>(self.default_sample_size)?;
        writer.write_u32::<BigEndian>(self.default_sample_flags)?;

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mp4box::BoxHeader;
    use std::io::Cursor;

    #[test]
    fn test_trex() {
        let src_box = TrexBox {
            version: 0,
            flags: 0,
            track_id: 1,
            default_sample_description_index: 1,
            default_sample_duration: 1000,
            default_sample_size: 0,
            default_sample_flags: 65536,
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::TrexBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = TrexBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
