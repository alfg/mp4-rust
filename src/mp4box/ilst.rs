use std::borrow::Cow;
use std::collections::HashMap;
use std::io::{Read, Seek};

use byteorder::ByteOrder;
use serde::Serialize;

use crate::mp4box::data::DataBox;
use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct IlstBox {
    pub items: HashMap<MetadataKey, IlstItemBox>,
}

impl IlstBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::IlstBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        for item in self.items.values() {
            size += item.get_size();
        }
        size
    }
}

impl Mp4Box for IlstBox {
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
        let s = format!("item_count={}", self.items.len());
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for IlstBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut items = HashMap::new();

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader { name, size: s } = header;

            match name {
                BoxType::NameBox => {
                    items.insert(MetadataKey::Title, IlstItemBox::read_box(reader, s)?);
                }
                BoxType::DayBox => {
                    items.insert(MetadataKey::Year, IlstItemBox::read_box(reader, s)?);
                }
                BoxType::CovrBox => {
                    items.insert(MetadataKey::Poster, IlstItemBox::read_box(reader, s)?);
                }
                BoxType::DescBox => {
                    items.insert(MetadataKey::Summary, IlstItemBox::read_box(reader, s)?);
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }

            current = reader.seek(SeekFrom::Current(0))?;
        }

        skip_bytes_to(reader, start + size)?;

        Ok(IlstBox { items })
    }
}

impl<W: Write> WriteBox<&mut W> for IlstBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        for (key, value) in &self.items {
            let name = match key {
                MetadataKey::Title => BoxType::NameBox,
                MetadataKey::Year => BoxType::DayBox,
                MetadataKey::Poster => BoxType::CovrBox,
                MetadataKey::Summary => BoxType::DescBox,
            };
            BoxHeader::new(name, value.get_size()).write(writer)?;
            value.data.write_box(writer)?;
        }
        Ok(size)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct IlstItemBox {
    pub data: DataBox,
}

impl IlstItemBox {
    fn get_size(&self) -> u64 {
        HEADER_SIZE + self.data.box_size()
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for IlstItemBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut data = None;

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader { name, size: s } = header;

            match name {
                BoxType::DataBox => {
                    data = Some(DataBox::read_box(reader, s)?);
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }

            current = reader.seek(SeekFrom::Current(0))?;
        }

        if data.is_none() {
            return Err(Error::BoxNotFound(BoxType::DataBox));
        }

        skip_bytes_to(reader, start + size)?;

        Ok(IlstItemBox {
            data: data.unwrap(),
        })
    }
}

impl<'a> Metadata<'a> for IlstBox {
    fn title(&self) -> Option<Cow<str>> {
        self.items.get(&MetadataKey::Title).map(item_to_str)
    }

    fn year(&self) -> Option<u32> {
        self.items.get(&MetadataKey::Year).and_then(item_to_u32)
    }

    fn poster(&self) -> Option<&[u8]> {
        self.items.get(&MetadataKey::Poster).map(item_to_bytes)
    }

    fn summary(&self) -> Option<Cow<str>> {
        self.items.get(&MetadataKey::Summary).map(item_to_str)
    }
}

fn item_to_bytes(item: &IlstItemBox) -> &[u8] {
    &item.data.data
}

fn item_to_str(item: &IlstItemBox) -> Cow<str> {
    String::from_utf8_lossy(&item.data.data)
}

fn item_to_u32(item: &IlstItemBox) -> Option<u32> {
    match item.data.data_type {
        DataType::Binary if item.data.data.len() == 4 => Some(BigEndian::read_u32(&item.data.data)),
        DataType::Text => String::from_utf8_lossy(&item.data.data).parse::<u32>().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mp4box::BoxHeader;
    use std::io::Cursor;

    #[test]
    fn test_ilst() {
        let src_year = IlstItemBox {
            data: DataBox {
                data_type: DataType::Text,
                data: b"test_year".to_vec(),
            },
        };
        let src_box = IlstBox {
            items: [
                (MetadataKey::Title, IlstItemBox::default()),
                (MetadataKey::Year, src_year),
                (MetadataKey::Poster, IlstItemBox::default()),
                (MetadataKey::Summary, IlstItemBox::default()),
            ]
            .into(),
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::IlstBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = IlstBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }

    #[test]
    fn test_ilst_empty() {
        let src_box = IlstBox::default();
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::IlstBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = IlstBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
