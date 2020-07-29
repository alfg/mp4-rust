use std::io::{BufReader, SeekFrom, Seek, Read, BufWriter, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::*;


#[derive(Debug, Default)]
pub struct ElstBox {
    pub version: u8,
    pub flags: u32,
    pub entry_count: u32,
    pub entries: Vec<ElstEntry>,
}

#[derive(Debug, Default)]
pub struct ElstEntry {
    pub segment_duration: u64,
    pub media_time: u64,
    pub media_rate: u16,
    pub media_rate_fraction: u16,
}

impl Mp4Box for ElstBox {
    fn box_type(&self) -> BoxType {
        BoxType::ElstBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE;
        if self.version == 1 {
            size += self.entry_count as u64 * 20;
        } else {
            assert_eq!(self.version, 0);
            size += self.entry_count as u64 * 12;
        }
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for ElstBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0))?; // Current cursor position.

        let (version, flags) = read_box_header_ext(reader)?;

        let entry_count = reader.read_u32::<BigEndian>()?;
        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            let (segment_duration, media_time)
                = if version == 1 {
                    (
                        reader.read_u64::<BigEndian>()?,
                        reader.read_u64::<BigEndian>()?,
                    )
                } else {
                    (
                        reader.read_u32::<BigEndian>()? as u64,
                        reader.read_u32::<BigEndian>()? as u64,
                    )
                };

            let entry = ElstEntry{
                segment_duration,
                media_time,
                media_rate: reader.read_u16::<BigEndian>()?,
                media_rate_fraction: reader.read_u16::<BigEndian>()?,
            };
            entries.push(entry);
        }
        skip_read(reader, current, size)?;

        Ok(ElstBox {
            version,
            flags,
            entry_count,
            entries,
        })
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for ElstBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        assert_eq!(self.entry_count as usize, self.entries.len());
        writer.write_u32::<BigEndian>(self.entry_count)?;
        for entry in self.entries.iter() {
            if self.version == 1 {
                writer.write_u64::<BigEndian>(entry.segment_duration)?;
                writer.write_u64::<BigEndian>(entry.media_time)?;
            } else {
                writer.write_u32::<BigEndian>(entry.segment_duration as u32)?;
                writer.write_u32::<BigEndian>(entry.media_time as u32)?;
            }
            writer.write_u16::<BigEndian>(entry.media_rate)?;
            writer.write_u16::<BigEndian>(entry.media_rate_fraction)?;
        }

        Ok(size)
    }
}
