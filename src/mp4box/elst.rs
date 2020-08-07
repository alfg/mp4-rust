use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ElstBox {
    pub version: u8,
    pub flags: u32,
    pub entries: Vec<ElstEntry>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ElstEntry {
    pub segment_duration: u64,
    pub media_time: u64,
    pub media_rate: u16,
    pub media_rate_fraction: u16,
}

impl Mp4Box for ElstBox {
    fn box_type() -> BoxType {
        BoxType::ElstBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE + 4;
        if self.version == 1 {
            size += self.entries.len() as u64 * 20;
        } else {
            assert_eq!(self.version, 0);
            size += self.entries.len() as u64 * 12;
        }
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for ElstBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        let entry_count = reader.read_u32::<BigEndian>()?;
        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            let (segment_duration, media_time) = if version == 1 {
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

            let entry = ElstEntry {
                segment_duration,
                media_time,
                media_rate: reader.read_u16::<BigEndian>()?,
                media_rate_fraction: reader.read_u16::<BigEndian>()?,
            };
            entries.push(entry);
        }

        skip_bytes_to(reader, start + size)?;

        Ok(ElstBox {
            version,
            flags,
            entries,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for ElstBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(Self::box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        writer.write_u32::<BigEndian>(self.entries.len() as u32)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mp4box::BoxHeader;
    use std::io::Cursor;

    #[test]
    fn test_elst32() {
        let src_box = ElstBox {
            version: 0,
            flags: 0,
            entries: vec![ElstEntry {
                segment_duration: 634634,
                media_time: 0,
                media_rate: 1,
                media_rate_fraction: 0,
            }],
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::ElstBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = ElstBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }

    #[test]
    fn test_elst64() {
        let src_box = ElstBox {
            version: 1,
            flags: 0,
            entries: vec![ElstEntry {
                segment_duration: 634634,
                media_time: 0,
                media_rate: 1,
                media_rate_fraction: 0,
            }],
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::ElstBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = ElstBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
