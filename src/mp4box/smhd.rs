use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq)]
pub struct SmhdBox {
    pub version: u8,
    pub flags: u32,
    pub balance: FixedPointI8,
}

impl Default for SmhdBox {
    fn default() -> Self {
        SmhdBox {
            version: 0,
            flags: 0,
            balance: FixedPointI8::new_raw(0),
        }
    }
}

impl Mp4Box for SmhdBox {
    fn box_type() -> BoxType {
        BoxType::SmhdBox
    }

    fn box_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 4
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for SmhdBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        let balance = FixedPointI8::new_raw(reader.read_i16::<BigEndian>()?);

        skip_bytes_to(reader, start + size)?;

        Ok(SmhdBox {
            version,
            flags,
            balance,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for SmhdBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(Self::box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        writer.write_i16::<BigEndian>(self.balance.raw_value())?;
        writer.write_u16::<BigEndian>(0)?; // reserved

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mp4box::BoxHeader;
    use std::io::Cursor;

    #[test]
    fn test_smhd() {
        let src_box = SmhdBox {
            version: 0,
            flags: 0,
            balance: FixedPointI8::new_raw(-1),
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::SmhdBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = SmhdBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
