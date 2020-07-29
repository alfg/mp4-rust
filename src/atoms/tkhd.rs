use std::io::{BufReader, Seek, SeekFrom, Read, BufWriter, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use num_rational::Ratio;

use crate::*;


#[derive(Debug, PartialEq)]
pub struct TkhdBox {
    pub version: u8,
    pub flags: u32,
    pub creation_time: u64,
    pub modification_time: u64,
    pub track_id: u32,
    pub duration: u64,
    pub layer:  u16,
    pub alternate_group: u16,
    pub volume: Ratio<u16>,
    pub matrix: Matrix,
    pub width: u32,
    pub height: u32,
}

impl Default for TkhdBox {
    fn default() -> Self {
        TkhdBox {
            version: 0,
            flags: 0,
            creation_time: 0,
            modification_time: 0,
            track_id: 0,
            duration: 0,
            layer: 0,
            alternate_group: 0,
            volume: Ratio::new_raw(0x0100, 0x100),
            matrix: Matrix::default(),
            width: 0,
            height: 0,
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Matrix {
    pub a: i32,
    pub b: i32,
    pub u: i32,
    pub c: i32,
    pub d: i32,
    pub v: i32,
    pub x: i32,
    pub y: i32,
    pub w: i32,
}

impl Mp4Box for TkhdBox {
    fn box_type(&self) -> BoxType {
        BoxType::TkhdBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE;
        if self.version == 1 {
            size += 32;
        } else {
            assert_eq!(self.version, 0);
            size += 20;
        }
        size += 60;
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for TkhdBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0))?; // Current cursor position.

        let (version, flags) = read_box_header_ext(reader)?;

        let (creation_time, modification_time, track_id, _, duration)
            = if version == 1 {
                (
                    reader.read_u64::<BigEndian>()?,
                    reader.read_u64::<BigEndian>()?,
                    reader.read_u32::<BigEndian>()?,
                    reader.read_u32::<BigEndian>()?,
                    reader.read_u64::<BigEndian>()?,
                )
        } else {
                assert_eq!(version, 0);
                (
                    reader.read_u32::<BigEndian>()? as u64,
                    reader.read_u32::<BigEndian>()? as u64,
                    reader.read_u32::<BigEndian>()?,
                    reader.read_u32::<BigEndian>()?,
                    reader.read_u32::<BigEndian>()? as u64,
                )
        };
        reader.read_u64::<BigEndian>()?; // reserved
        let layer = reader.read_u16::<BigEndian>()?;
        let alternate_group = reader.read_u16::<BigEndian>()?;
        let volume_numer = reader.read_u16::<BigEndian>()?;
        let volume = Ratio::new_raw(volume_numer, 0x100);

        reader.read_u16::<BigEndian>()?; // reserved
        let matrix = Matrix{
            a: reader.read_i32::<byteorder::LittleEndian>()?,
            b: reader.read_i32::<BigEndian>()?,
            u: reader.read_i32::<BigEndian>()?,
            c: reader.read_i32::<BigEndian>()?,
            d: reader.read_i32::<BigEndian>()?,
            v: reader.read_i32::<BigEndian>()?,
            x: reader.read_i32::<BigEndian>()?,
            y: reader.read_i32::<BigEndian>()?,
            w: reader.read_i32::<BigEndian>()?,
        };

        let width = reader.read_u32::<BigEndian>()? >> 16;
        let height = reader.read_u32::<BigEndian>()? >> 16;

        skip_read(reader, current, size)?;

        Ok(TkhdBox {
            version,
            flags,
            creation_time,
            modification_time,
            track_id,
            duration,
            layer,
            alternate_group,
            volume,
            matrix,
            width,
            height,
        })
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for TkhdBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        if self.version == 1 {
            writer.write_u64::<BigEndian>(self.creation_time)?;
            writer.write_u64::<BigEndian>(self.modification_time)?;
            writer.write_u32::<BigEndian>(self.track_id)?;
            writer.write_u32::<BigEndian>(0)?; // reserved
            writer.write_u64::<BigEndian>(self.duration)?;
        } else {
            assert_eq!(self.version, 0);
            writer.write_u32::<BigEndian>(self.creation_time as u32)?;
            writer.write_u32::<BigEndian>(self.modification_time as u32)?;
            writer.write_u32::<BigEndian>(self.track_id)?;
            writer.write_u32::<BigEndian>(0)?; // reserved
            writer.write_u32::<BigEndian>(self.duration as u32)?;
        }

        writer.write_u64::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(self.layer)?;
        writer.write_u16::<BigEndian>(self.alternate_group)?;
        writer.write_u16::<BigEndian>(*self.volume.numer())?;

        writer.write_u16::<BigEndian>(0)?; // reserved

        writer.write_i32::<byteorder::LittleEndian>(self.matrix.a)?;
        writer.write_i32::<BigEndian>(self.matrix.b)?;
        writer.write_i32::<BigEndian>(self.matrix.u)?;
        writer.write_i32::<BigEndian>(self.matrix.c)?;
        writer.write_i32::<BigEndian>(self.matrix.d)?;
        writer.write_i32::<BigEndian>(self.matrix.v)?;
        writer.write_i32::<BigEndian>(self.matrix.x)?;
        writer.write_i32::<BigEndian>(self.matrix.y)?;
        writer.write_i32::<BigEndian>(self.matrix.w)?;

        writer.write_u32::<BigEndian>(self.width << 16)?;
        writer.write_u32::<BigEndian>(self.height << 16)?;

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::read_box_header;
    use std::io::Cursor;

    #[test]
    fn test_tkhd32() {
        let src_box = TkhdBox {
            version: 0,
            flags: 0,
            creation_time: 100,
            modification_time: 200,
            track_id: 1,
            duration: 634634,
            layer: 0,
            alternate_group: 0,
            volume: Ratio::new_raw(0x0100, 0x100),
            matrix: Matrix {
                a: 0x00010000,
                b: 0,
                u: 0,
                c: 0,
                d: 0x00010000,
                v: 0,
                x: 0,
                y: 0,
                w: 0x40000000,
            },
            width: 512,
            height: 288,
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
            assert_eq!(header.name, BoxType::TkhdBox);
            assert_eq!(src_box.box_size(), header.size);

            let dst_box = TkhdBox::read_box(&mut reader, header.size).unwrap();

            assert_eq!(src_box, dst_box);
        }
    }

    #[test]
    fn test_tkhd64() {
        let src_box = TkhdBox {
            version: 1,
            flags: 0,
            creation_time: 100,
            modification_time: 200,
            track_id: 1,
            duration: 634634,
            layer: 0,
            alternate_group: 0,
            volume: Ratio::new_raw(0x0100, 0x100),
            matrix: Matrix {
                a: 0x00010000,
                b: 0,
                u: 0,
                c: 0,
                d: 0x00010000,
                v: 0,
                x: 0,
                y: 0,
                w: 0x40000000,
            },
            width: 512,
            height: 288,
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
            assert_eq!(header.name, BoxType::TkhdBox);
            assert_eq!(src_box.box_size(), header.size);

            let dst_box = TkhdBox::read_box(&mut reader, header.size).unwrap();

            assert_eq!(src_box, dst_box);
        }
    }
}
