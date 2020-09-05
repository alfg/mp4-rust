use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Tx3gBox {
    pub data_reference_index: u16,
}

impl Default for Tx3gBox {
    fn default() -> Self {
        Tx3gBox {
            data_reference_index: 0,
        }
    }
}

impl Tx3gBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::Tx3gBox
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE + 8
    }
}

impl Mp4Box for Tx3gBox {
    fn box_type(&self) -> BoxType {
        return self.get_type();
    }

    fn box_size(&self) -> u64 {
        return self.get_size();
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for Tx3gBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        reader.read_u32::<BigEndian>()?; // reserved
        reader.read_u16::<BigEndian>()?; // reserved
        let data_reference_index = reader.read_u16::<BigEndian>()?;

        skip_bytes_to(reader, start + size)?;

        Ok(Tx3gBox {
            data_reference_index,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for Tx3gBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        writer.write_u32::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(self.data_reference_index)?;

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mp4box::BoxHeader;
    use std::io::Cursor;

    #[test]
    fn test_tx3g() {
        let src_box = Tx3gBox {
            data_reference_index: 1,
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::Tx3gBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = Tx3gBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
