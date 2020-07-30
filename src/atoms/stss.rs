use std::io::{BufReader, Seek, Read, BufWriter, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::*;


#[derive(Debug, Default, PartialEq)]
pub struct StssBox {
    pub version: u8,
    pub flags: u32,
    pub entries: Vec<u32>,
}

impl Mp4Box for StssBox {
    fn box_type() -> BoxType {
        BoxType::StssBox
    }

    fn box_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 4 + (4 * self.entries.len() as u64)
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for StssBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let start = get_box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        let entry_count = reader.read_u32::<BigEndian>()?;
        let mut entries = Vec::with_capacity(entry_count as usize);
        for _i in 0..entry_count {
            let sample_number = reader.read_u32::<BigEndian>()?;
            entries.push(sample_number);
        }

        skip_read_to(reader, start + size)?;

        Ok(StssBox {
            version,
            flags,
            entries,
        })
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for StssBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(Self::box_type(), size).write_box(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        writer.write_u32::<BigEndian>(self.entries.len() as u32)?;
        for sample_number in self.entries.iter() {
            writer.write_u32::<BigEndian>(*sample_number)?;
        }

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::read_box_header;
    use std::io::Cursor;

    #[test]
    fn test_stss() {
        let src_box = StssBox {
            version: 0,
            flags: 0,
            entries: vec![1, 61, 121, 181, 241, 301, 361, 421, 481],
        };
        let mut buf = Vec::new();
        {
            let mut writer = BufWriter::new(&mut buf);
            src_box.write_box(&mut writer).unwrap();
        }
        assert_eq!(buf.len(), src_box.box_size() as usize);

        {
            let mut reader = BufReader::new(Cursor::new(&buf));
            let header = read_box_header(&mut reader).unwrap();
            assert_eq!(header.name, BoxType::StssBox);
            assert_eq!(src_box.box_size(), header.size);

            let dst_box = StssBox::read_box(&mut reader, header.size).unwrap();

            assert_eq!(src_box, dst_box);
        }
    }
}
