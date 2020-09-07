use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, Write};
use serde::{Serialize};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Tx3gBox {
    pub data_reference_index: u16,
    pub display_flags: u32,
    pub horizontal_justification: i8,
    pub vertical_justification: i8,
    pub bg_color_rgba: RgbaColor,
    pub box_record: [i16; 4],
    pub style_record: [u8; 12],
}

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct RgbaColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8
}

impl Default for Tx3gBox {
    fn default() -> Self {
        Tx3gBox {
            data_reference_index: 0,
            display_flags: 0,
            horizontal_justification: 1,
            vertical_justification: -1,
            bg_color_rgba: RgbaColor{
                red: 0,
                green: 0,
                blue: 0,
                alpha: 255,
            },
            box_record: [0, 0, 0, 0],
            style_record: [0, 0, 0, 0, 0, 1, 0, 16, 255, 255, 255, 255],
        }
    }
}

impl Tx3gBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::Tx3gBox
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE + 6 + 32 
    }
}

impl Mp4Box for Tx3gBox {
    fn box_type(&self) -> BoxType {
        return self.get_type();
    }

    fn box_size(&self) -> u64 {
        return self.get_size();
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("data_reference_index={} horizontal_justification={} vertical_justification={} rgba={}{}{}{}",
            self.data_reference_index, self.horizontal_justification,
            self.vertical_justification, self.bg_color_rgba.red,
            self.bg_color_rgba.green, self.bg_color_rgba.blue, self.bg_color_rgba.alpha);
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for Tx3gBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        reader.read_u32::<BigEndian>()?; // reserved
        reader.read_u16::<BigEndian>()?; // reserved
        let data_reference_index = reader.read_u16::<BigEndian>()?;

        let display_flags = reader.read_u32::<BigEndian>()?;
        let horizontal_justification = reader.read_i8()?;
        let vertical_justification = reader.read_i8()?;
        let bg_color_rgba = RgbaColor {
            red: reader.read_u8()?,
            green: reader.read_u8()?,
            blue: reader.read_u8()?,
            alpha: reader.read_u8()?,
        };
        let box_record: [i16; 4] = [
            reader.read_i16::<BigEndian>()?,
            reader.read_i16::<BigEndian>()?,
            reader.read_i16::<BigEndian>()?,
            reader.read_i16::<BigEndian>()?,
        ];
        let style_record: [u8; 12] = [
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u8()?,
        ];

        skip_bytes_to(reader, start + size)?;

        Ok(Tx3gBox {
            data_reference_index,
            display_flags,
            horizontal_justification,
            vertical_justification,
            bg_color_rgba,
            box_record,
            style_record,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for Tx3gBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        writer.write_u32::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(self.data_reference_index)?;
        writer.write_u32::<BigEndian>(self.display_flags)?;
        writer.write_i8(self.horizontal_justification)?;
        writer.write_i8(self.vertical_justification)?;
        writer.write_u8(self.bg_color_rgba.red)?;
        writer.write_u8(self.bg_color_rgba.green)?;
        writer.write_u8(self.bg_color_rgba.blue)?;
        writer.write_u8(self.bg_color_rgba.alpha)?;
        for n in 0..4 {
            writer.write_i16::<BigEndian>(self.box_record[n])?;
        }
        for n in 0..12 {
            writer.write_u8(self.style_record[n])?;
        }

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mp4box::BoxHeader;
    use std::io::Cursor;

    #[test]
    fn test_tx3g() {
        let src_box = Tx3gBox {
            data_reference_index: 1,
            display_flags: 0,
            horizontal_justification: 1,
            vertical_justification: -1,
            bg_color_rgba: RgbaColor{
                red: 0,
                green: 0,
                blue: 0,
                alpha: 255,
            },
            box_record: [0, 0, 0, 0],
            style_record: [0, 0, 0, 0, 0, 1, 0, 16, 255, 255, 255, 255],
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::Tx3gBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = Tx3gBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
