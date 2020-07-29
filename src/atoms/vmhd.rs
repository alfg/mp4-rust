use std::io::{BufReader, SeekFrom, Seek, Read, BufWriter, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::*;


#[derive(Debug, Default, PartialEq)]
pub struct VmhdBox {
    pub version: u8,
    pub flags: u32,
    pub graphics_mode: u16,
    pub op_color: RgbColor,
}

#[derive(Debug, Default, PartialEq)]
pub struct RgbColor {
    pub red: u16,
    pub green: u16,
    pub blue: u16,
}

impl Mp4Box for VmhdBox {
    fn box_type(&self) -> BoxType {
        BoxType::VmhdBox
    }

    fn box_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 8
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for VmhdBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0))?; // Current cursor position.

        let (version, flags) = read_box_header_ext(reader)?;

        let graphics_mode = reader.read_u16::<BigEndian>()?;
        let op_color = RgbColor {
            red: reader.read_u16::<BigEndian>()?,
            green: reader.read_u16::<BigEndian>()?,
            blue: reader.read_u16::<BigEndian>()?,
        };
        skip_read(reader, current, size)?;

        Ok(VmhdBox {
            version,
            flags,
            graphics_mode,
            op_color,
        })
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for VmhdBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        writer.write_u16::<BigEndian>(self.graphics_mode)?;
        writer.write_u16::<BigEndian>(self.op_color.red)?;
        writer.write_u16::<BigEndian>(self.op_color.green)?;
        writer.write_u16::<BigEndian>(self.op_color.blue)?;

        Ok(size)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::read_box_header;
    use std::io::Cursor;

    #[test]
    fn test_vmhd() {
        let src_box = VmhdBox {
            version: 0,
            flags: 1,
            graphics_mode: 0,
            op_color: RgbColor { red: 0, green: 0, blue: 0},
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
            assert_eq!(header.name, BoxType::VmhdBox);
            assert_eq!(src_box.box_size(), header.size);

            let dst_box = VmhdBox::read_box(&mut reader, header.size).unwrap();

            assert_eq!(src_box, dst_box);
        }
    }
}
