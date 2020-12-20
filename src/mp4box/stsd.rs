use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
#[cfg(feature = "use_serde")]
use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::avcn::{Avc1Variant, Avc2Variant, Avc3Variant};
use crate::mp4box::*;
use crate::mp4box::{avcn::AvcNBox, hev1::Hev1Box, mp4a::Mp4aBox, tx3g::Tx3gBox};

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "use_serde", derive(Serialize))]
pub struct StsdBox {
    pub version: u8,
    pub flags: u32,

    #[cfg_attr(feature = "use_serde", serde(skip_serializing_if = "Option::is_none"))]
    pub avc1: Option<AvcNBox<Avc1Variant>>,

    #[cfg_attr(feature = "use_serde", serde(skip_serializing_if = "Option::is_none"))]
    pub avc2: Option<AvcNBox<Avc2Variant>>,

    #[cfg_attr(feature = "use_serde", serde(skip_serializing_if = "Option::is_none"))]
    pub avc3: Option<AvcNBox<Avc3Variant>>,

    #[cfg_attr(feature = "use_serde", serde(skip_serializing_if = "Option::is_none"))]
    pub hev1: Option<Hev1Box>,

    #[cfg_attr(feature = "use_serde", serde(skip_serializing_if = "Option::is_none"))]
    pub mp4a: Option<Mp4aBox>,

    #[cfg_attr(feature = "use_serde", serde(skip_serializing_if = "Option::is_none"))]
    pub tx3g: Option<Tx3gBox>,
}

impl StsdBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::StsdBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE + 4;
        if let Some(ref avc1) = self.avc1 {
            size += avc1.box_size();
        } else if let Some(ref avc2) = self.avc2 {
            size += avc2.box_size();
        } else if let Some(ref avc3) = self.avc3 {
            size += avc3.box_size();
        } else if let Some(ref hev1) = self.hev1 {
            size += hev1.box_size();
        } else if let Some(ref mp4a) = self.mp4a {
            size += mp4a.box_size();
        } else if let Some(ref tx3g) = self.tx3g {
            size += tx3g.box_size();
        }
        size
    }
}

impl Mp4Box for StsdBox {
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
        let s = format!("");
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for StsdBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        reader.read_u32::<BigEndian>()?; // XXX entry_count

        let mut avc1 = None;
        let mut avc2 = None;
        let mut avc3 = None;
        let mut hev1 = None;
        let mut mp4a = None;
        let mut tx3g = None;

        // Get box header.
        let header = BoxHeader::read(reader)?;
        let BoxHeader { name, size: s } = header;

        match name {
            BoxType::Avc1Box => {
                avc1 = Some(AvcNBox::read_box(reader, s)?);
            }
            BoxType::Avc2Box => {
                avc2 = Some(AvcNBox::read_box(reader, s)?);
            }
            BoxType::Avc3Box => {
                avc3 = Some(AvcNBox::read_box(reader, s)?);
            }
            BoxType::Hev1Box => {
                hev1 = Some(Hev1Box::read_box(reader, s)?);
            }
            BoxType::Mp4aBox => {
                mp4a = Some(Mp4aBox::read_box(reader, s)?);
            }
            BoxType::Tx3gBox => {
                tx3g = Some(Tx3gBox::read_box(reader, s)?);
            }
            _ => {}
        }

        skip_bytes_to(reader, start + size)?;

        Ok(StsdBox {
            version,
            flags,
            avc1,
            avc2,
            avc3,
            hev1,
            mp4a,
            tx3g,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for StsdBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        writer.write_u32::<BigEndian>(1)?; // entry_count

        if let Some(ref avc1) = self.avc1 {
            avc1.write_box(writer)?;
        } else if let Some(ref avc2) = self.avc2 {
            avc2.write_box(writer)?;
        } else if let Some(ref avc3) = self.avc3 {
            avc3.write_box(writer)?;
        } else if let Some(ref hev1) = self.hev1 {
            hev1.write_box(writer)?;
        } else if let Some(ref mp4a) = self.mp4a {
            mp4a.write_box(writer)?;
        } else if let Some(ref tx3g) = self.tx3g {
            tx3g.write_box(writer)?;
        }

        Ok(size)
    }
}
