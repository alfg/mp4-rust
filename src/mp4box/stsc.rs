use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;
use std::io::{Read, Seek, Write};
use std::mem::size_of;

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct StscBox {
    pub version: u8,
    pub flags: u32,

    #[serde(skip_serializing)]
    pub entries: Vec<StscEntry>,
}

impl StscBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::StscBox
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 4 + (12 * self.entries.len() as u64)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct StscEntry {
    pub first_chunk: u32,
    pub samples_per_chunk: u32,
    pub sample_description_index: u32,
    pub first_sample: u32,
}

impl Mp4Box for StscBox {
    fn box_type(&self) -> BoxType {
        self.get_type()
    }

    fn box_size(&self) -> u64 {
        self.get_size()
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("entries={}", self.entries.len());
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for StscBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        let header_size = HEADER_SIZE + HEADER_EXT_SIZE;
        let other_size = size_of::<u32>(); // entry_count
        let entry_size = size_of::<u32>() + size_of::<u32>() + size_of::<u32>(); // first_chunk + samples_per_chunk + sample_description_index
        let entry_count = reader.read_u32::<BigEndian>()?;
        if u64::from(entry_count)
            > size
                .saturating_sub(header_size)
                .saturating_sub(other_size as u64)
                / entry_size as u64
        {
            return Err(Error::InvalidData(
                "stsc entry_count indicates more entries than could fit in the box",
            ));
        }
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
                sample_id = next_entry
                    .first_chunk
                    .checked_sub(first_chunk)
                    .and_then(|n| n.checked_mul(samples_per_chunk))
                    .and_then(|n| n.checked_add(sample_id))
                    .ok_or(Error::InvalidData(
                        "attempt to calculate stsc sample_id with overflow",
                    ))?;
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
        BoxHeader::new(self.box_type(), size).write(writer)?;

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
