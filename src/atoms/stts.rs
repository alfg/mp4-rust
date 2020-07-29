use std::io::{BufReader, SeekFrom, Seek, Read, BufWriter, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::*;


#[derive(Debug, Default, PartialEq)]
pub struct SttsBox {
    pub version: u8,
    pub flags: u32,
    pub entries: Vec<SttsEntry>,
}

#[derive(Debug, Default, PartialEq)]
pub struct SttsEntry {
    pub sample_count: u32,
    pub sample_delta: u32,
}

impl Mp4Box for SttsBox {
    fn box_type(&self) -> BoxType {
        BoxType::SttsBox
    }

    fn box_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 4 + (8 * self.entries.len() as u64)
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for SttsBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0))?; // Current cursor position.

        let (version, flags) = read_box_header_ext(reader)?;

        let entry_count = reader.read_u32::<BigEndian>()?;
        let mut entries = Vec::with_capacity(entry_count as usize);
        for _i in 0..entry_count {
            let entry = SttsEntry {
                sample_count: reader.read_u32::<BigEndian>()?,
                sample_delta: reader.read_u32::<BigEndian>()?,
            };
            entries.push(entry);
        }
        skip_read(reader, current, size)?;

        Ok(SttsBox {
            version,
            flags,
            entries,
        })
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for SttsBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        writer.write_u32::<BigEndian>(self.entries.len() as u32)?;
        for entry in self.entries.iter() {
            writer.write_u32::<BigEndian>(entry.sample_count)?;
            writer.write_u32::<BigEndian>(entry.sample_delta)?;
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
    fn test_stts32() {
        let src_box = SttsBox {
            version: 0,
            flags: 0,
            entries: vec![
                SttsEntry {sample_count: 29726, sample_delta: 1024},
                SttsEntry {sample_count: 1, sample_delta: 512},
            ],
        };
        let mut buf = Vec::new();
        {
            let mut writer = BufWriter::new(&mut buf);
            src_box.write_box(&mut writer).unwrap();
        }
        assert_eq!(buf.len(), src_box.box_size() as usize);

        {
            let mut reader = BufReader::new(Cursor::new(&buf));
            let header = read_box_header(&mut reader, 0).unwrap();
            assert_eq!(header.name, BoxType::SttsBox);
            assert_eq!(src_box.box_size(), header.size);

            let dst_box = SttsBox::read_box(&mut reader, header.size).unwrap();

            assert_eq!(src_box, dst_box);
        }
    }

    #[test]
    fn test_stts64() {
        let src_box = SttsBox {
            version: 1,
            flags: 0,
            entries: vec![
                SttsEntry {sample_count: 29726, sample_delta: 1024},
                SttsEntry {sample_count: 1, sample_delta: 512},
            ],
        };
        let mut buf = Vec::new();
        {
            let mut writer = BufWriter::new(&mut buf);
            src_box.write_box(&mut writer).unwrap();
        }
        assert_eq!(buf.len(), src_box.box_size() as usize);

        {
            let mut reader = BufReader::new(Cursor::new(&buf));
            let header = read_box_header(&mut reader, 0).unwrap();
            assert_eq!(header.name, BoxType::SttsBox);
            assert_eq!(src_box.box_size(), header.size);

            let dst_box = SttsBox::read_box(&mut reader, header.size).unwrap();

            assert_eq!(src_box, dst_box);
        }
    }
}
