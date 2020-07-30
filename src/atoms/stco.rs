use std::io::{Seek, Read, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::*;
use crate::atoms::*;


#[derive(Debug, Default, PartialEq)]
pub struct StcoBox {
    pub version: u8,
    pub flags: u32,
    pub entries: Vec<u32>,
}

impl Mp4Box for StcoBox {
    fn box_type() -> BoxType {
        BoxType::StcoBox
    }

    fn box_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 4 + (4 * self.entries.len() as u64)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for StcoBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = get_box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        let entry_count = reader.read_u32::<BigEndian>()?;
        let mut entries = Vec::with_capacity(entry_count as usize);
        for _i in 0..entry_count {
            let chunk_offset = reader.read_u32::<BigEndian>()?;
            entries.push(chunk_offset);
        }

        skip_read_to(reader, start + size)?;

        Ok(StcoBox {
            version,
            flags,
            entries,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for StcoBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(Self::box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        writer.write_u32::<BigEndian>(self.entries.len() as u32)?;
        for chunk_offset in self.entries.iter() {
            writer.write_u32::<BigEndian>(*chunk_offset)?;
        }

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atoms::BoxHeader;
    use std::io::Cursor;

    #[test]
    fn test_stco() {
        let src_box = StcoBox {
            version: 0,
            flags: 0,
            entries: vec![267, 1970, 2535, 2803, 11843, 22223, 33584],
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::StcoBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = StcoBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
