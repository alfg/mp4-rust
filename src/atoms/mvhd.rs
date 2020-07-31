use std::io::{Seek, Read, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use num_rational::Ratio;

use crate::*;
use crate::atoms::*;


#[derive(Debug, PartialEq)]
pub struct MvhdBox {
    pub version: u8,
    pub flags: u32,
    pub creation_time: u64,
    pub modification_time: u64,
    pub timescale: u32,
    pub duration: u64,
    pub rate: Ratio<u32>,
}

impl Default for MvhdBox {
    fn default() -> Self {
        MvhdBox {
            version: 0,
            flags: 0,
            creation_time: 0,
            modification_time: 0,
            timescale: 1000,
            duration: 0,
            rate: Ratio::new_raw(0x00010000, 0x10000),
        }
    }
}

impl Mp4Box for MvhdBox {
    fn box_type() -> BoxType {
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

impl<R: Read + Seek> ReadBox<&mut R> for MvhdBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = get_box_start(reader)?;

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
        let numer = reader.read_u32::<BigEndian>()?;
        let rate = Ratio::new_raw(numer, 0x10000);

        skip_read_to(reader, start + size)?;

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

impl<W: Write> WriteBox<&mut W> for MvhdBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(Self::box_type(), size).write(writer)?;

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
        writer.write_u32::<BigEndian>(*self.rate.numer())?;

        // XXX volume, ...
        skip_write(writer, 76)?;

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atoms::BoxHeader;
    use std::io::Cursor;

    #[test]
    fn test_mvhd32() {
        let src_box = MvhdBox {
            version: 0,
            flags: 0,
            creation_time: 100,
            modification_time: 200,
            timescale: 1000,
            duration: 634634,
            rate: Ratio::new_raw(0x00010000, 0x10000),
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::MvhdBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = MvhdBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }

    #[test]
    fn test_mvhd64() {
        let src_box = MvhdBox {
            version: 1,
            flags: 0,
            creation_time: 100,
            modification_time: 200,
            timescale: 1000,
            duration: 634634,
            rate: Ratio::new_raw(0x00010000, 0x10000),
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::MvhdBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = MvhdBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
