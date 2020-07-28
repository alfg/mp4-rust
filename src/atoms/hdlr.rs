use std::io::{BufReader, SeekFrom, Seek, Read, BufWriter, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::{Result};
use crate::{FourCC, BoxType, BoxHeader, Mp4Box, ReadBox, WriteBox};
use crate::{HEADER_SIZE, HEADER_EXT_SIZE};
use crate::{read_box_header_ext, write_box_header_ext, skip_read};


#[derive(Debug, Default)]
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
        let current = reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

        let (version, flags) = read_box_header_ext(reader).unwrap();

        reader.read_u32::<BigEndian>().unwrap(); // pre-defined
        let handler = reader.read_u32::<BigEndian>().unwrap();

        let n = reader.seek(SeekFrom::Current(12)).unwrap(); // 12 bytes reserved.

        let buf_size = (size - (n - current)) - HEADER_SIZE;
        let mut buf = vec![0u8; buf_size as usize];
        reader.read_exact(&mut buf).unwrap();

        let handler_string = match String::from_utf8(buf) {
            Ok(t) => {
                assert_eq!(t.len(), buf_size as usize);
                t
            },
            _ => String::from("null"),
        };

        skip_read(reader, current, size);

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

        writer.write_u32::<BigEndian>(0).unwrap(); // pre-defined
        writer.write_u32::<BigEndian>((&self.handler_type).into()).unwrap();

        // 12 bytes reserved
        for _ in 0..3 {
            writer.write_u32::<BigEndian>(0).unwrap();
        }

        writer.write(self.name.as_bytes()).unwrap();
        writer.write_u8(0).unwrap();

        Ok(size)
    }
}
