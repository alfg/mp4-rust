use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, Write};
use serde::{Serialize};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MehdBox {
    pub version: u8,
    pub flags: u32,
    pub fragment_duration: u64,
}

impl MehdBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::MehdBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE;

        if self.version == 1 {
            size += 8;
        } else if self.version == 0 {
            size += 4;
        }
        size
    }
}

impl Default for MehdBox {
    fn default() -> Self {
        MehdBox {
            version: 0,
            flags: 0,
            fragment_duration: 0,
        }
    }
}

impl Mp4Box for MehdBox {
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
        let s = format!("fragment_duration={}", self.fragment_duration);
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for MehdBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        let fragment_duration = if version == 1 {
            reader.read_u64::<BigEndian>()?
        } else if version == 0 {
            reader.read_u32::<BigEndian>()? as u64
        } else {
            return Err(Error::InvalidData("version must be 0 or 1"));
        };
        skip_bytes_to(reader, start + size)?;

        Ok(MehdBox {
            version,
            flags,
            fragment_duration,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for MehdBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        if self.version == 1 {
            writer.write_u64::<BigEndian>(self.fragment_duration)?;
        } else if self.version == 0 {
            writer.write_u32::<BigEndian>(self.fragment_duration as u32)?;
        } else {
            return Err(Error::InvalidData("version must be 0 or 1"));
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
    fn test_mehd32() {
        let src_box = MehdBox {
            version: 0,
            flags: 0,
            fragment_duration: 32,
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::MehdBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = MehdBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }

    #[test]
    fn test_mehd64() {
        let src_box = MehdBox {
            version: 0,
            flags: 0,
            fragment_duration: 30439936,
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::MehdBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = MehdBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
