use std::io::{Seek, Read, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use num_rational::Ratio;

use crate::*;
use crate::atoms::*;


#[derive(Debug, PartialEq)]
pub struct SmhdBox {
    pub version: u8,
    pub flags: u32,
    pub balance: Ratio<i16>,
}

impl Default for SmhdBox {
    fn default() -> Self {
        SmhdBox {
            version: 0,
            flags: 0,
            balance: Ratio::new_raw(0, 0x100),
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
        let start = get_box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        let balance_numer = reader.read_i16::<BigEndian>()?;
        let balance = Ratio::new_raw(balance_numer, 0x100);

        skip_read_to(reader, start + size)?;

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

        writer.write_i16::<BigEndian>(*self.balance.numer())?;
        writer.write_u16::<BigEndian>(0)?; // reserved

        Ok(size)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::atoms::BoxHeader;
    use std::io::Cursor;

    #[test]
    fn test_smhd() {
        let src_box = SmhdBox {
            version: 0,
            flags: 0,
            balance: Ratio::new_raw(-0x100, 0x100),
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
