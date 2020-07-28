use std::io::{BufReader, Seek, SeekFrom, Read, BufWriter, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::{Result};
use crate::{BoxType, BoxHeader, Mp4Box, ReadBox, WriteBox};
use crate::{HEADER_SIZE, HEADER_EXT_SIZE};
use crate::{read_box_header_ext, write_box_header_ext, skip_read};


#[derive(Debug, Default, PartialEq)]
pub struct TkhdBox {
    pub version: u8,
    pub flags: u32,
    pub creation_time: u64,
    pub modification_time: u64,
    pub track_id: u32,
    pub duration: u64,
    pub layer:  u16,
    pub alternate_group: u16,
    pub volume: u16,
    pub matrix: Matrix,
    pub width: u32,
    pub height: u32,
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
        let current = reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

        let (version, flags) = read_box_header_ext(reader).unwrap();

        let (creation_time, modification_time, track_id, _, duration)
            = if version == 1 {
                (
                    reader.read_u64::<BigEndian>().unwrap(),
                    reader.read_u64::<BigEndian>().unwrap(),
                    reader.read_u32::<BigEndian>().unwrap(),
                    reader.read_u32::<BigEndian>().unwrap(),
                    reader.read_u64::<BigEndian>().unwrap(),
                )
        } else {
                assert_eq!(version, 0);
                (
                    reader.read_u32::<BigEndian>().unwrap() as u64,
                    reader.read_u32::<BigEndian>().unwrap() as u64,
                    reader.read_u32::<BigEndian>().unwrap(),
                    reader.read_u32::<BigEndian>().unwrap(),
                    reader.read_u32::<BigEndian>().unwrap() as u64,
                )
        };
        reader.read_u64::<BigEndian>().unwrap(); // reserved
        let layer = reader.read_u16::<BigEndian>().unwrap();
        let alternate_group = reader.read_u16::<BigEndian>().unwrap();
        let volume = reader.read_u16::<BigEndian>().unwrap();

        reader.read_u16::<BigEndian>().unwrap(); // reserved
        let matrix = Matrix{
            a: reader.read_i32::<byteorder::LittleEndian>().unwrap(),
            b: reader.read_i32::<BigEndian>().unwrap(),
            u: reader.read_i32::<BigEndian>().unwrap(),
            c: reader.read_i32::<BigEndian>().unwrap(),
            d: reader.read_i32::<BigEndian>().unwrap(),
            v: reader.read_i32::<BigEndian>().unwrap(),
            x: reader.read_i32::<BigEndian>().unwrap(),
            y: reader.read_i32::<BigEndian>().unwrap(),
            w: reader.read_i32::<BigEndian>().unwrap(),
        };

        let width = reader.read_u32::<BigEndian>().unwrap() >> 16;
        let height = reader.read_u32::<BigEndian>().unwrap() >> 16;

        skip_read(reader, current, size);

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
            writer.write_u64::<BigEndian>(self.creation_time).unwrap();
            writer.write_u64::<BigEndian>(self.modification_time).unwrap();
            writer.write_u32::<BigEndian>(self.track_id).unwrap();
            writer.write_u32::<BigEndian>(0).unwrap(); // reserved
            writer.write_u64::<BigEndian>(self.duration).unwrap();
        } else {
            assert_eq!(self.version, 0);
            writer.write_u32::<BigEndian>(self.creation_time as u32).unwrap();
            writer.write_u32::<BigEndian>(self.modification_time as u32).unwrap();
            writer.write_u32::<BigEndian>(self.track_id).unwrap();
            writer.write_u32::<BigEndian>(0).unwrap(); // reserved
            writer.write_u32::<BigEndian>(self.duration as u32).unwrap();
        }

        writer.write_u64::<BigEndian>(0).unwrap(); // reserved
        writer.write_u16::<BigEndian>(self.layer).unwrap();
        writer.write_u16::<BigEndian>(self.alternate_group).unwrap();
        writer.write_u16::<BigEndian>(self.volume).unwrap();

        writer.write_u16::<BigEndian>(0).unwrap(); // reserved

        writer.write_i32::<byteorder::LittleEndian>(self.matrix.a).unwrap();
        writer.write_i32::<BigEndian>(self.matrix.b).unwrap();
        writer.write_i32::<BigEndian>(self.matrix.u).unwrap();
        writer.write_i32::<BigEndian>(self.matrix.c).unwrap();
        writer.write_i32::<BigEndian>(self.matrix.d).unwrap();
        writer.write_i32::<BigEndian>(self.matrix.v).unwrap();
        writer.write_i32::<BigEndian>(self.matrix.x).unwrap();
        writer.write_i32::<BigEndian>(self.matrix.y).unwrap();
        writer.write_i32::<BigEndian>(self.matrix.w).unwrap();

        writer.write_u32::<BigEndian>(self.width << 16).unwrap();
        writer.write_u32::<BigEndian>(self.height << 16).unwrap();

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::read_box_header;
    use std::io::Cursor;

    #[test]
    fn test_tkhd() {
        let src_box = TkhdBox {
            version: 0,
            flags: 0,
            creation_time: 100,
            modification_time: 200,
            track_id: 1,
            duration: 634634,
            layer: 0,
            alternate_group: 0,
            volume: 0x0100,
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
