use std::borrow::Cow;
use std::collections::HashMap;
use std::io::{Read, Seek};

use byteorder::ByteOrder;
use serde::Serialize;

use crate::mp4box::data::DataBox;
use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct IlstBox {
    pub items: HashMap<MetadataKey, IlstItemBox>,
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

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct IlstItemBox {
    pub data: DataBox,
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
