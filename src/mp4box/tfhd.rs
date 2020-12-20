use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
#[cfg(feature = "use_serde")]
use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "use_serde", derive(Serialize))]
pub struct TfhdBox {
    pub version: u8,
    pub track_id: u32,

    #[cfg_attr(feature = "use_serde", serde(skip_serializing_if = "Option::is_none"))]
    pub base_data_offset: Option<u64>,
    #[cfg_attr(feature = "use_serde", serde(skip_serializing_if = "Option::is_none"))]
    pub sample_description_index: Option<u32>,
    #[cfg_attr(feature = "use_serde", serde(skip_serializing_if = "Option::is_none"))]
    pub default_sample_duration: Option<u32>,
    #[cfg_attr(feature = "use_serde", serde(skip_serializing_if = "Option::is_none"))]
    pub default_sample_size: Option<u32>,
    #[cfg_attr(feature = "use_serde", serde(skip_serializing_if = "Option::is_none"))]
    pub default_sample_flags: Option<u32>,

    pub duration_is_empty: bool,
    pub default_base_is_moof: bool,
}

impl Default for TfhdBox {
    fn default() -> Self {
        TfhdBox {
            version: 0,
            track_id: 0,
            base_data_offset: Some(0),
            sample_description_index: None,
            default_sample_duration: None,
            default_sample_size: None,
            default_sample_flags: None,
            duration_is_empty: false,
            default_base_is_moof: false,
        }
    }
}

impl TfhdBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::TfhdBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE;
        size += 4;
        if self.base_data_offset.is_some() {
            size += 8;
        }
        if self.sample_description_index.is_some() {
            size += 4;
        }
        if self.default_sample_duration.is_some() {
            size += 4;
        }
        if self.default_sample_size.is_some() {
            size += 4;
        }
        if self.default_sample_flags.is_some() {
            size += 4;
        }
        size
    }
}

impl Mp4Box for TfhdBox {
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
        let s = format!("track_id={}", self.track_id);
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for TfhdBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        let base_data_offset_present = (flags & 0x000001) != 0;
        let sample_description_index_present = (flags & 0x000002) != 0;
        let default_sample_duration_present = (flags & 0x000008) != 0;
        let default_sample_size_present = (flags & 0x000010) != 0;
        let default_sample_flags_present = (flags & 0x000020) != 0;
        let duration_is_empty = (flags & 0x010000) != 0;
        let default_base_is_moof = (flags & 0x020000) != 0;

        let track_id = reader.read_u32::<BigEndian>()?;

        let base_data_offset = if base_data_offset_present {
            Some(reader.read_u64::<BigEndian>()?)
        } else {
            None
        };
        let sample_description_index = if sample_description_index_present {
            Some(reader.read_u32::<BigEndian>()?)
        } else {
            None
        };
        let default_sample_duration = if default_sample_duration_present {
            Some(reader.read_u32::<BigEndian>()?)
        } else {
            None
        };
        let default_sample_size = if default_sample_size_present {
            Some(reader.read_u32::<BigEndian>()?)
        } else {
            None
        };
        let default_sample_flags = if default_sample_flags_present {
            Some(reader.read_u32::<BigEndian>()?)
        } else {
            None
        };

        skip_bytes_to(reader, start + size)?;

        Ok(TfhdBox {
            version,
            track_id,

            base_data_offset,
            sample_description_index,
            default_sample_duration,
            default_sample_size,
            default_sample_flags,

            duration_is_empty,
            default_base_is_moof,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for TfhdBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        let mut flags = 0;
        if self.base_data_offset.is_some() {
            flags |= 0x000001;
        }
        if self.sample_description_index.is_some() {
            flags |= 0x000002;
        }
        if self.default_sample_duration.is_some() {
            flags |= 0x000008;
        }
        if self.default_sample_size.is_some() {
            flags |= 0x000010;
        }
        if self.default_sample_flags.is_some() {
            flags |= 0x000020;
        }
        if self.duration_is_empty {
            flags |= 0x010000;
        }
        if self.default_base_is_moof {
            flags |= 0x020000;
        }

        write_box_header_ext(writer, self.version, flags)?;

        writer.write_u32::<BigEndian>(self.track_id)?;

        if let Some(val) = self.base_data_offset {
            writer.write_u64::<BigEndian>(val)?;
        }
        if let Some(val) = self.sample_description_index {
            writer.write_u32::<BigEndian>(val)?;
        }
        if let Some(val) = self.default_sample_duration {
            writer.write_u32::<BigEndian>(val)?;
        }
        if let Some(val) = self.default_sample_size {
            writer.write_u32::<BigEndian>(val)?;
        }
        if let Some(val) = self.default_sample_flags {
            writer.write_u32::<BigEndian>(val)?;
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
    fn test_tfhd() {
        let src_box = TfhdBox {
            version: 0,
            track_id: 1,
            base_data_offset: Some(0),
            sample_description_index: None,
            default_sample_duration: None,
            default_sample_size: None,
            default_sample_flags: None,
            duration_is_empty: false,
            default_base_is_moof: false,
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::TfhdBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = TfhdBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
