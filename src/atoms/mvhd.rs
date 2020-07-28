use std::io::{BufReader, SeekFrom, Seek, Read, BufWriter, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::{Result};
use crate::{BoxType, BoxHeader, Mp4Box, ReadBox, WriteBox};
use crate::{HEADER_SIZE, HEADER_EXT_SIZE};
use crate::{read_box_header_ext, write_box_header_ext, skip_read, skip_write};


#[derive(Debug, Default, PartialEq)]
pub struct MvhdBox {
    pub version: u8,
    pub flags: u32,
    pub creation_time: u64,
    pub modification_time: u64,
    pub timescale: u32,
    pub duration: u64,
    pub rate: u32,
}

impl Mp4Box for MvhdBox {
    fn box_type(&self) -> BoxType {
        BoxType::MvhdBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE;
        if self.version == 1 {
            size += 28;
        } else {
            assert_eq!(self.version, 0);
            size += 16;
        }
        size += 80;
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for MvhdBox {
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
        let rate = reader.read_u32::<BigEndian>().unwrap();
        skip_read(reader, current, size);

        Ok(MvhdBox{
            version,
            flags,
            creation_time,
            modification_time,
            timescale,
            duration,
            rate,
        })
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for MvhdBox {
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
        writer.write_u32::<BigEndian>(self.rate).unwrap();

        // XXX volume, ...
        skip_write(writer, 76);

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::read_box_header;
    use std::io::Cursor;

    #[test]
    fn test_mvhd() {
        let src_box = MvhdBox {
            version: 0,
            flags: 0,
            creation_time: 100,
            modification_time: 200,
            timescale: 1000,
            duration: 634634,
            rate: 0x00010000,
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
            assert_eq!(header.name, BoxType::MvhdBox);
            assert_eq!(src_box.box_size(), header.size);

            let dst_box = MvhdBox::read_box(&mut reader, header.size).unwrap();

            assert_eq!(src_box, dst_box);
        }
    }
}
