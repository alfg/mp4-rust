use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, Write};
use serde::{Serialize};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct HdlrBox {
    pub version: u8,
    pub flags: u32,
    pub handler_type: FourCC,
    pub name: String,
}

impl HdlrBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::HdlrBox
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 20 + self.name.len() as u64 + 1
    }
}

impl Mp4Box for HdlrBox {
    fn box_type(&self) -> BoxType {
        return self.get_type();
    }

    fn box_size(&self) -> u64 {
        return self.get_size();
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("handler_type={} name={}", self.handler_type.to_string(), self.name);
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for HdlrBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        reader.read_u32::<BigEndian>()?; // pre-defined
        let handler = reader.read_u32::<BigEndian>()?;

        skip_bytes(reader, 12)?; // reserved

        let buf_size = size - HEADER_SIZE - HEADER_EXT_SIZE - 20 - 1;
        let mut buf = vec![0u8; buf_size as usize];
        reader.read_exact(&mut buf)?;

        let handler_string = match String::from_utf8(buf) {
            Ok(t) => {
                if t.len() != buf_size as usize {
                    return Err(Error::InvalidData("string too small"))
                }
                t
            }
            _ => String::from("null"),
        };

        skip_bytes_to(reader, start + size)?;

        Ok(HdlrBox {
            version,
            flags,
            handler_type: From::from(handler),
            name: handler_string,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for HdlrBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        writer.write_u32::<BigEndian>(0)?; // pre-defined
        writer.write_u32::<BigEndian>((&self.handler_type).into())?;

        // 12 bytes reserved
        for _ in 0..3 {
            writer.write_u32::<BigEndian>(0)?;
        }

        writer.write(self.name.as_bytes())?;
        writer.write_u8(0)?;

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mp4box::BoxHeader;
    use std::io::Cursor;

    #[test]
    fn test_hdlr() {
        let src_box = HdlrBox {
            version: 0,
            flags: 0,
            handler_type: str::parse::<FourCC>("vide").unwrap(),
            name: String::from("VideoHandler"),
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::HdlrBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = HdlrBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
