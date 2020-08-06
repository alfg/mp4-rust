use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StscBox {
    pub version: u8,
    pub flags: u32,
    pub entries: Vec<StscEntry>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StscEntry {
    pub first_chunk: u32,
    pub samples_per_chunk: u32,
    pub sample_description_index: u32,
    pub first_sample: u32,
}

impl Mp4Box for StscBox {
    fn box_type() -> BoxType {
        BoxType::StscBox
    }

    fn box_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 4 + (12 * self.entries.len() as u64)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for StscBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        let entry_count = reader.read_u32::<BigEndian>()?;
        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            let entry = StscEntry {
                first_chunk: reader.read_u32::<BigEndian>()?,
                samples_per_chunk: reader.read_u32::<BigEndian>()?,
                sample_description_index: reader.read_u32::<BigEndian>()?,
                first_sample: 0,
            };
            entries.push(entry);
        }

        let mut sample_id = 1;
        for i in 0..entry_count {
            let (first_chunk, samples_per_chunk) = {
                let mut entry = entries.get_mut(i as usize).unwrap();
                entry.first_sample = sample_id;
                (entry.first_chunk, entry.samples_per_chunk)
            };
            if i < entry_count - 1 {
                let next_entry = entries.get(i as usize + 1).unwrap();
                sample_id += (next_entry.first_chunk - first_chunk) * samples_per_chunk;
            }
        }

        skip_bytes_to(reader, start + size)?;

        Ok(StscBox {
            version,
            flags,
            entries,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for StscBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(Self::box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        writer.write_u32::<BigEndian>(self.entries.len() as u32)?;
        for entry in self.entries.iter() {
            writer.write_u32::<BigEndian>(entry.first_chunk)?;
            writer.write_u32::<BigEndian>(entry.samples_per_chunk)?;
            writer.write_u32::<BigEndian>(entry.sample_description_index)?;
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
    fn test_stsc() {
        let src_box = StscBox {
            version: 0,
            flags: 0,
            entries: vec![
                StscEntry {
                    first_chunk: 1,
                    samples_per_chunk: 1,
                    sample_description_index: 1,
                    first_sample: 1,
                },
                StscEntry {
                    first_chunk: 19026,
                    samples_per_chunk: 14,
                    sample_description_index: 1,
                    first_sample: 19026,
                },
            ],
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::StscBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = StscBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
