use std::io::{BufReader, SeekFrom, Seek, Read, BufWriter, Write};
use std::char::{decode_utf16, REPLACEMENT_CHARACTER};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::*;


#[derive(Debug, PartialEq)]
pub struct MdhdBox {
    pub version: u8,
    pub flags: u32,
    pub creation_time: u64,
    pub modification_time: u64,
    pub timescale: u32,
    pub duration: u64,
    pub language: String,
}

impl Default for MdhdBox {
    fn default() -> Self {
        MdhdBox {
            version: 0,
            flags: 0,
            creation_time: 0,
            modification_time: 0,
            timescale: 1000,
            duration: 0,
            language: String::from("und"),
        }
    }
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
        let current = reader.seek(SeekFrom::Current(0))?; // Current cursor position.

        let (version, flags) = read_box_header_ext(reader)?;

        let (creation_time, modification_time, timescale, duration)
            = if version  == 1 {
                (
                    reader.read_u64::<BigEndian>()?,
                    reader.read_u64::<BigEndian>()?,
                    reader.read_u32::<BigEndian>()?,
                    reader.read_u64::<BigEndian>()?,
                )
            } else {
                assert_eq!(version, 0);
                (
                    reader.read_u32::<BigEndian>()? as u64,
                    reader.read_u32::<BigEndian>()? as u64,
                    reader.read_u32::<BigEndian>()?,
                    reader.read_u32::<BigEndian>()? as u64,
                )
            };
        let language_code = reader.read_u16::<BigEndian>()?;
        let language = get_language_string(language_code);
        skip_read(reader, current, size)?;

        Ok(MdhdBox {
            version,
            flags,
            creation_time,
            modification_time,
            timescale,
            duration,
            language,
        })
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for MdhdBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        if self.version == 1 {
            writer.write_u64::<BigEndian>(self.creation_time)?;
            writer.write_u64::<BigEndian>(self.modification_time)?;
            writer.write_u32::<BigEndian>(self.timescale)?;
            writer.write_u64::<BigEndian>(self.duration)?;
        } else {
            assert_eq!(self.version, 0);
            writer.write_u32::<BigEndian>(self.creation_time as u32)?;
            writer.write_u32::<BigEndian>(self.modification_time as u32)?;
            writer.write_u32::<BigEndian>(self.timescale)?;
            writer.write_u32::<BigEndian>(self.duration as u32)?;
        }

        let language_code = get_language_code(&self.language);
        writer.write_u16::<BigEndian>(language_code)?;
        writer.write_u16::<BigEndian>(0)?; // pre-defined

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

fn get_language_code(language: &str) -> u16 {
    let mut lang = language.encode_utf16();
    let mut code = (lang.next().unwrap_or(0) & 0x1F) << 10;
    code += (lang.next().unwrap_or(0) & 0x1F) << 5;
    code += lang.next().unwrap_or(0) & 0x1F;
    code
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::read_box_header;
    use std::io::Cursor;

    fn test_language_code(lang: &str) {
        let code = get_language_code(lang);
        let lang2 = get_language_string(code);
        assert_eq!(lang, lang2);
    }

    #[test]
    fn test_language_codes() {
        test_language_code("und");
        test_language_code("eng");
        test_language_code("kor");
    }

    #[test]
    fn test_mdhd32() {
        let src_box = MdhdBox {
            version: 0,
            flags: 0,
            creation_time: 100,
            modification_time: 200,
            timescale: 48000,
            duration: 30439936,
            language: String::from("und"),
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
            assert_eq!(header.name, BoxType::MdhdBox);
            assert_eq!(src_box.box_size(), header.size);

            let dst_box = MdhdBox::read_box(&mut reader, header.size).unwrap();

            assert_eq!(src_box, dst_box);
        }
    }

    #[test]
    fn test_mdhd64() {
        let src_box = MdhdBox {
            version: 0,
            flags: 0,
            creation_time: 100,
            modification_time: 200,
            timescale: 48000,
            duration: 30439936,
            language: String::from("eng"),
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
            assert_eq!(header.name, BoxType::MdhdBox);
            assert_eq!(src_box.box_size(), header.size);

            let dst_box = MdhdBox::read_box(&mut reader, header.size).unwrap();

            assert_eq!(src_box, dst_box);
        }
    }
}
