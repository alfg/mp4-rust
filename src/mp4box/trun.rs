use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
#[cfg(feature = "use_serde")]
use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;
use crate::SampleFlags;

#[cfg_attr(feature = "use_serde", derive(Serialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TrunBox {
    pub version: u8,
    pub sample_count: u32,

    pub data_offset: Option<i32>,
    pub first_sample_flags: Option<SampleFlags>,

    #[cfg_attr(feature = "use_serde", serde(skip_serializing))]
    pub sample_durations: Option<Vec<u32>>,
    #[cfg_attr(feature = "use_serde", serde(skip_serializing))]
    pub sample_sizes: Option<Vec<u32>>,
    #[cfg_attr(feature = "use_serde", serde(skip_serializing))]
    pub sample_flags: Option<Vec<SampleFlags>>,
    #[cfg_attr(feature = "use_serde", serde(skip_serializing))]
    pub sample_composition_time_offsets: Option<Vec<i64>>,
}

impl TrunBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::TrunBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE;
        size += 4;
        if self.data_offset.is_some() {
            size += 4;
        }
        if self.first_sample_flags.is_some() {
            size += 4;
        }
        if self.sample_durations.is_some() {
            size += 4 * self.sample_count as u64;
        }
        if self.sample_sizes.is_some() {
            size += 4 * self.sample_count as u64;
        }
        if self.sample_flags.is_some() {
            size += 4 * self.sample_count as u64;
        }
        if self.sample_composition_time_offsets.is_some() {
            size += 4 * self.sample_count as u64;
        }
        size
    }
}

impl Mp4Box for TrunBox {
    fn box_type(&self) -> BoxType {
        return self.get_type();
    }

    fn box_size(&self) -> u64 {
        return self.get_size();
    }

    #[cfg(feature = "use_serde")]
    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("sample_size={}", self.sample_count);
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for TrunBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        let data_offset_present = (flags & 0x000001) != 0;
        let first_sample_flags_present = (flags & 0x000004) != 0;
        let sample_duration_present = (flags & 0x000100) != 0;
        let sample_size_present = (flags & 0x000200) != 0;
        let sample_flags_present = (flags & 0x000400) != 0;
        let sample_composition_time_offsets_present = (flags & 0x000800) != 0;

        let sample_count = reader.read_u32::<BigEndian>()?;

        let data_offset = if data_offset_present {
            Some(reader.read_i32::<BigEndian>()?)
        } else {
            None
        };
        let first_sample_flags = if first_sample_flags_present {
            Some(SampleFlags::new(reader.read_u32::<BigEndian>()?))
        } else {
            None
        };

        let mut sample_durations = if sample_duration_present {
            Some(Vec::with_capacity(sample_count as usize))
        } else {
            None
        };
        let mut sample_sizes = if sample_size_present {
            Some(Vec::with_capacity(sample_count as usize))
        } else {
            None
        };
        let mut sample_flags = if sample_flags_present {
            Some(Vec::with_capacity(sample_count as usize))
        } else {
            None
        };
        let mut sample_composition_time_offsets = if sample_composition_time_offsets_present {
            Some(Vec::with_capacity(sample_count as usize))
        } else {
            None
        };

        for _ in 0..sample_count {
            if let Some(vec) = &mut sample_durations {
                vec.push(reader.read_u32::<BigEndian>()?);
            }
            if let Some(vec) = &mut sample_sizes {
                vec.push(reader.read_u32::<BigEndian>()?);
            }
            if let Some(vec) = &mut sample_flags {
                vec.push(SampleFlags::new(reader.read_u32::<BigEndian>()?));
            }
            if let Some(vec) = &mut sample_composition_time_offsets {
                if version == 0 {
                    vec.push(reader.read_u32::<BigEndian>()?.into());
                } else {
                    vec.push(reader.read_i32::<BigEndian>()?.into());
                }
            }
        }

        skip_bytes_to(reader, start + size)?;

        Ok(TrunBox {
            version,
            sample_count,
            data_offset,
            first_sample_flags,
            sample_durations,
            sample_sizes,
            sample_flags,
            sample_composition_time_offsets,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for TrunBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        let mut flags = 0;
        if self.data_offset.is_some() {
            flags |= 0x000001;
        }
        if self.first_sample_flags.is_some() {
            flags |= 0x000004;
        }
        if let Some(vec) = &self.sample_durations {
            assert_eq!(self.sample_count, vec.len() as u32);
            flags |= 0x000100;
        }
        if let Some(vec) = &self.sample_sizes {
            assert_eq!(self.sample_count, vec.len() as u32);
            flags |= 0x000200;
        }
        if let Some(vec) = &self.sample_flags {
            assert_eq!(self.sample_count, vec.len() as u32);
            flags |= 0x000400;
        }
        if let Some(vec) = &self.sample_composition_time_offsets {
            assert_eq!(self.sample_count, vec.len() as u32);
            flags |= 0x000800;
        }
        write_box_header_ext(writer, self.version, flags)?;

        writer.write_u32::<BigEndian>(self.sample_count)?;

        if let Some(val) = self.data_offset {
            writer.write_i32::<BigEndian>(val)?;
        }
        if let Some(val) = self.first_sample_flags {
            writer.write_u32::<BigEndian>(val.to_u32())?;
        }

        for n in 0..(self.sample_count as usize) {
            if let Some(vec) = &self.sample_durations {
                writer.write_u32::<BigEndian>(vec[n])?;
            }
            if let Some(vec) = &self.sample_sizes {
                writer.write_u32::<BigEndian>(vec[n])?;
            }
            if let Some(vec) = &self.sample_flags {
                writer.write_u32::<BigEndian>(vec[n].to_u32())?;
            }
            if let Some(vec) = &self.sample_composition_time_offsets {
                if self.version == 0 {
                    writer.write_u32::<BigEndian>(vec[n].try_into().unwrap())?;
                } else {
                    writer.write_i32::<BigEndian>(vec[n].try_into().unwrap())?;
                }
            }
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
    fn test_trun_same_size() {
        let src_box = TrunBox {
            version: 0,
            sample_count: 0,
            data_offset: Some(0),
            first_sample_flags: None,
            sample_durations: None,
            sample_sizes: Some(vec![]),
            sample_flags: None,
            sample_composition_time_offsets: None,
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::TrunBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = TrunBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }

    #[test]
    fn test_trun_many_sizes() {
        let src_box = TrunBox {
            version: 0,
            data_offset: Some(0),
            first_sample_flags: None,
            sample_count: 9,
            sample_durations: None,
            sample_sizes: vec![1165, 11, 11, 8545, 10126, 10866, 9643, 9351, 7730],
            sample_flags: None,
            sample_composition_time_offsets: None,
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::TrunBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = TrunBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
