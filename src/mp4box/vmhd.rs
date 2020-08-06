use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct VmhdBox {
    pub version: u8,
    pub flags: u32,
    pub graphics_mode: u16,
    pub op_color: RgbColor,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RgbColor {
    pub red: u16,
    pub green: u16,
    pub blue: u16,
}

impl Mp4Box for VmhdBox {
    fn box_type() -> BoxType {
        BoxType::VmhdBox
    }

    fn box_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 8
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for VmhdBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        let graphics_mode = reader.read_u16::<BigEndian>()?;
        let op_color = RgbColor {
            red: reader.read_u16::<BigEndian>()?,
            green: reader.read_u16::<BigEndian>()?,
            blue: reader.read_u16::<BigEndian>()?,
        };

        skip_bytes_to(reader, start + size)?;

        Ok(VmhdBox {
            version,
            flags,
            graphics_mode,
            op_color,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for VmhdBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(Self::box_type(), size).write(writer)?;

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
    use crate::mp4box::BoxHeader;
    use std::io::Cursor;

    #[test]
    fn test_vmhd() {
        let src_box = VmhdBox {
            version: 0,
            flags: 1,
            graphics_mode: 0,
            op_color: RgbColor {
                red: 0,
                green: 0,
                blue: 0,
            },
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::VmhdBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = VmhdBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
