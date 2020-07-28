use std::io::{BufReader, SeekFrom, Seek, Read, BufWriter, Write};
use std::char::{decode_utf16, REPLACEMENT_CHARACTER};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::{Result};
use crate::{BoxType, BoxHeader, Mp4Box, ReadBox, WriteBox};
use crate::{HEADER_SIZE, HEADER_EXT_SIZE};
use crate::{read_box_header_ext, write_box_header_ext, skip_read};


#[derive(Debug, Default)]
pub struct MdhdBox {
    pub version: u8,
    pub flags: u32,
    pub creation_time: u64,
    pub modification_time: u64,
    pub timescale: u32,
    pub duration: u64,
    pub language: u16,
    pub language_string: String,
}

impl Mp4Box for MdhdBox {
    fn box_type(&self) -> BoxType {
        BoxType::MdhdBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE;

        if self.version == 1 {
            size += 28;
        } else {
            assert_eq!(self.version, 0);
            size += 16;
        }
        size += 4;
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for MdhdBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

        let (version, flags) = read_box_header_ext(reader).unwrap();

        let (creation_time, modification_time, timescale, duration)
            = if version  == 1 {
                (
                    reader.read_u64::<BigEndian>().unwrap(),
                    reader.read_u64::<BigEndian>().unwrap(),
                    reader.read_u32::<BigEndian>().unwrap(),
                    reader.read_u64::<BigEndian>().unwrap(),
                )
            } else {
                assert_eq!(version, 0);
                (
                    reader.read_u32::<BigEndian>().unwrap() as u64,
                    reader.read_u32::<BigEndian>().unwrap() as u64,
                    reader.read_u32::<BigEndian>().unwrap(),
                    reader.read_u32::<BigEndian>().unwrap() as u64,
                )
            };
        let language = reader.read_u16::<BigEndian>().unwrap();
        let language_string = get_language_string(language);
        skip_read(reader, current, size);

        Ok(MdhdBox {
            version,
            flags,
            creation_time,
            modification_time,
            timescale,
            duration,
            language,
            language_string,
        })
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for MdhdBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        if self.version == 1 {
            writer.write_u64::<BigEndian>(self.creation_time).unwrap();
            writer.write_u64::<BigEndian>(self.modification_time).unwrap();
            writer.write_u32::<BigEndian>(self.timescale).unwrap();
            writer.write_u64::<BigEndian>(self.duration).unwrap();
        } else {
            assert_eq!(self.version, 0);
            writer.write_u32::<BigEndian>(self.creation_time as u32).unwrap();
            writer.write_u32::<BigEndian>(self.modification_time as u32).unwrap();
            writer.write_u32::<BigEndian>(self.timescale).unwrap();
            writer.write_u32::<BigEndian>(self.duration as u32).unwrap();
        }

        writer.write_u16::<BigEndian>(self.language).unwrap();
        writer.write_u16::<BigEndian>(0).unwrap(); // pre-defined

        Ok(size)
    }
}

fn get_language_string(language: u16) -> String {
    let mut lang: [u16; 3] = [0; 3];

    lang[0] = ((language >> 10) & 0x1F) + 0x60;
    lang[1] = ((language >> 5) & 0x1F) + 0x60;
    lang[2] = ((language) & 0x1F) + 0x60;

    // Decode utf-16 encoded bytes into a string.
    let lang_str = decode_utf16(lang.iter().cloned())
        .map(|r| r.unwrap_or(REPLACEMENT_CHARACTER))
        .collect::<String>();

    return lang_str;
}
