use std::io::{BufReader, SeekFrom, Seek, Read, BufWriter, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::*;


#[derive(Debug, Default, PartialEq)]
pub struct HdlrBox {
    pub version: u8,
    pub flags: u32,
    pub handler_type: FourCC,
    pub name: String,
}

impl Mp4Box for HdlrBox {
    fn box_type(&self) -> BoxType {
        BoxType::HdlrBox
    }

    fn box_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 20 + self.name.len() as u64 + 1
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for HdlrBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0))?; // Current cursor position.

        let (version, flags) = read_box_header_ext(reader)?;

        reader.read_u32::<BigEndian>()?; // pre-defined
        let handler = reader.read_u32::<BigEndian>()?;

        let n = reader.seek(SeekFrom::Current(12))?; // 12 bytes reserved.

        let buf_size = (size - (n - current)) - HEADER_SIZE - 1;
        let mut buf = vec![0u8; buf_size as usize];
        reader.read_exact(&mut buf)?;

        let handler_string = match String::from_utf8(buf) {
            Ok(t) => {
                assert_eq!(t.len(), buf_size as usize);
                t
            },
            _ => String::from("null"),
        };

        skip_read(reader, current, size)?;

        Ok(HdlrBox {
            version,
            flags,
            handler_type: From::from(handler),
            name: handler_string,
        })
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for HdlrBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

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
    use crate::read_box_header;
    use std::io::Cursor;

    #[test]
    fn test_hdlr() {
        let src_box = HdlrBox {
            version: 0,
            flags: 0,
            handler_type: FourCC::from("vide"),
            name: String::from("VideoHandler"),
        };
        let mut buf = Vec::new();
        {
            let mut writer = BufWriter::new(&mut buf);
            src_box.write_box(&mut writer).unwrap();
        }
        assert_eq!(buf.len(), src_box.box_size() as usize);

        {
            let mut reader = BufReader::new(Cursor::new(&buf));
            let header = read_box_header(&mut reader, 0).unwrap();
            assert_eq!(header.name, BoxType::HdlrBox);
            assert_eq!(src_box.box_size(), header.size);

            let dst_box = HdlrBox::read_box(&mut reader, header.size).unwrap();

            assert_eq!(src_box, dst_box);
        }
    }
}
