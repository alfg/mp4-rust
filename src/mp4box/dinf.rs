use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct DinfBox {
    dref: DrefBox,
}

impl DinfBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::DinfBox
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE + self.dref.box_size()
    }
}

impl Mp4Box for DinfBox {
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
        let s = String::new();
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for DinfBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut dref = None;

        let mut current = reader.stream_position()?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader { name, size: s } = header;
            if s > size {
                return Err(Error::InvalidData(
                    "dinf box contains a box with a larger size than it",
                ));
            }

            match name {
                BoxType::DrefBox => {
                    dref = Some(DrefBox::read_box(reader, s)?);
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }

            current = reader.stream_position()?;
        }

        if dref.is_none() {
            return Err(Error::BoxNotFound(BoxType::DrefBox));
        }

        skip_bytes_to(reader, start + size)?;

        Ok(DinfBox {
            dref: dref.unwrap(),
        })
    }
}

impl<W: Write> WriteBox<&mut W> for DinfBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;
        self.dref.write_box(writer)?;
        Ok(size)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DrefBox {
    pub version: u8,
    pub flags: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<UrlBox>,
}

impl Default for DrefBox {
    fn default() -> Self {
        DrefBox {
            version: 0,
            flags: 0,
            url: Some(UrlBox::default()),
        }
    }
}

impl DrefBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::DrefBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE + 4;
        if let Some(ref url) = self.url {
            size += url.box_size();
        }
        size
    }
}

impl Mp4Box for DrefBox {
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
        let s = String::new();
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for DrefBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut current = reader.stream_position()?;

        let (version, flags) = read_box_header_ext(reader)?;
        let end = start + size;

        let mut url = None;

        let entry_count = reader.read_u32::<BigEndian>()?;
        for _i in 0..entry_count {
            if current >= end {
                break;
            }

            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader { name, size: s } = header;
            if s > size {
                return Err(Error::InvalidData(
                    "dinf box contains a box with a larger size than it",
                ));
            }

            match name {
                BoxType::UrlBox => {
                    url = Some(UrlBox::read_box(reader, s)?);
                }
                _ => {
                    skip_box(reader, s)?;
                }
            }

            current = reader.stream_position()?;
        }

        skip_bytes_to(reader, start + size)?;

        Ok(DrefBox {
            version,
            flags,
            url,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for DrefBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        writer.write_u32::<BigEndian>(1)?;

        if let Some(ref url) = self.url {
            url.write_box(writer)?;
        }

        Ok(size)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UrlBox {
    pub version: u8,
    pub flags: u32,
    pub location: String,
}

impl Default for UrlBox {
    fn default() -> Self {
        UrlBox {
            version: 0,
            flags: 1,
            location: String::default(),
        }
    }
}

impl UrlBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::UrlBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE;

        if !self.location.is_empty() {
            size += self.location.bytes().len() as u64 + 1;
        }

        size
    }
}

impl Mp4Box for UrlBox {
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
        let s = format!("location={}", self.location);
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for UrlBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        let location = if size.saturating_sub(HEADER_SIZE + HEADER_EXT_SIZE) > 0 {
            let buf_size = size - HEADER_SIZE - HEADER_EXT_SIZE - 1;
            let mut buf = vec![0u8; buf_size as usize];
            reader.read_exact(&mut buf)?;
            match String::from_utf8(buf) {
                Ok(t) => {
                    if t.len() != buf_size as usize {
                        return Err(Error::InvalidData("string too small"));
                    }
                    t
                }
                _ => String::default(),
            }
        } else {
            String::default()
        };

        skip_bytes_to(reader, start + size)?;

        Ok(UrlBox {
            version,
            flags,
            location,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for UrlBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        if !self.location.is_empty() {
            writer.write_all(self.location.as_bytes())?;
            writer.write_u8(0)?;
        }

        Ok(size)
    }
}
