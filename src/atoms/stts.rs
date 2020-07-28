use std::io::{BufReader, SeekFrom, Seek, Read, BufWriter, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::{Result};
use crate::{BoxType, BoxHeader, Mp4Box, ReadBox, WriteBox};
use crate::{HEADER_SIZE, HEADER_EXT_SIZE};
use crate::{read_box_header_ext, write_box_header_ext, skip_read};


#[derive(Debug, Default)]
pub struct SttsBox {
    pub version: u8,
    pub flags: u32,
    pub entry_count: u32,
    pub entries: Vec<SttsEntry>,
}

#[derive(Debug, Default)]
pub struct SttsEntry {
    pub sample_count: u32,
    pub sample_delta: u32,
}

impl Mp4Box for SttsBox {
    fn box_type(&self) -> BoxType {
        BoxType::SttsBox
    }

    fn box_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 4 + (8 * self.entry_count as u64)
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for SttsBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

        let (version, flags) = read_box_header_ext(reader).unwrap();

        let entry_count = reader.read_u32::<BigEndian>().unwrap();
        let mut entries = Vec::with_capacity(entry_count as usize);
        for _i in 0..entry_count {
            let entry = SttsEntry {
                sample_count: reader.read_u32::<BigEndian>().unwrap(),
                sample_delta: reader.read_u32::<BigEndian>().unwrap(),
            };
            entries.push(entry);
        }
        skip_read(reader, current, size);

        Ok(SttsBox {
            version,
            flags,
            entry_count,
            entries,
        })
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for SttsBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        writer.write_u32::<BigEndian>(self.entry_count).unwrap();
        for entry in self.entries.iter() {
            writer.write_u32::<BigEndian>(entry.sample_count).unwrap();
            writer.write_u32::<BigEndian>(entry.sample_delta).unwrap();
        }

        Ok(size)
    }
}
