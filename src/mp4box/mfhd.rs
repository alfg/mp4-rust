use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq)]
pub struct MfhdBox {
    pub version: u8,
    pub flags: u32,
    pub sequence_number: u32,
}

impl Default for MfhdBox {
    fn default() -> Self {
        MfhdBox {
            version: 0,
            flags: 0,
            sequence_number: 1,
        }
    }
}

impl MfhdBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::MfhdBox
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 4 
    }
}

impl Mp4Box for MfhdBox {
    fn box_type(&self) -> BoxType {
        return self.get_type();
    }

    fn box_size(&self) -> u64 {
        return self.get_size();
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for MfhdBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;
        let sequence_number = reader.read_u32::<BigEndian>()?;

        skip_bytes_to(reader, start + size)?;

        Ok(MfhdBox {
            version,
            flags,
            sequence_number,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for MfhdBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;
        writer.write_u32::<BigEndian>(self.sequence_number)?;

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mp4box::BoxHeader;
    use std::io::Cursor;

    #[test]
    fn test_mfhd() {
        let src_box = MfhdBox {
            version: 0,
            flags: 0,
            sequence_number: 1,
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::MfhdBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = MfhdBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
