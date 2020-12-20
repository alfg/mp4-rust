use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
#[cfg(feature = "use_serde")]
use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

pub enum TrackFlag {
    TrackEnabled = 0x000001,
    // TrackInMovie = 0x000002,
    // TrackInPreview = 0x000004,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "use_serde", derive(Serialize))]
pub struct TkhdBox {
    pub version: u8,
    pub flags: u32,
    pub creation_time: u64,
    pub modification_time: u64,
    pub track_id: u32,
    pub duration: u64,
    pub layer: u16,
    pub alternate_group: u16,

    pub volume: FixedPointU16,
    pub matrix: Matrix,

    pub width: FixedPointU32,
    pub height: FixedPointU32,
}

impl Default for TkhdBox {
    fn default() -> Self {
        TkhdBox {
            version: 0,
            flags: TrackFlag::TrackEnabled as u32,
            creation_time: 0,
            modification_time: 0,
            track_id: 0,
            duration: 0,
            layer: 0,
            alternate_group: 0,
            volume: FixedPointU16::new_whole(1),
            matrix: Matrix::default(),
            width: FixedPointU32::new_raw(0),
            height: FixedPointU32::new_raw(0),
        }
    }
}

impl TkhdBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::TkhdBox
    }

    pub fn get_size(&self) -> u64 {
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

    pub fn set_width(&mut self, width: u16) {
        self.width = FixedPointU32::new_whole(width as u32);
    }

    pub fn set_height(&mut self, height: u16) {
        self.height = FixedPointU32::new_whole(height as u32);
    }
}

impl Mp4Box for TkhdBox {
    fn box_type(&self) -> BoxType {
        return self.get_type();
    }

    fn box_size(&self) -> u64 {
        return self.get_size();
    }

    #[cfg(feature = "use_serde")]
    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!(
            "creation_time={} track_id={} duration={} layer={} volume={:?} width={:?} height={:?}",
            self.creation_time,
            self.track_id,
            self.duration,
            self.layer,
            self.volume,
            self.width,
            self.height
        );
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for TkhdBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        let (creation_time, modification_time, track_id, _, duration) = if version == 1 {
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
        let volume = FixedPointU16::new_raw(reader.read_u16::<BigEndian>()?);

        reader.read_u16::<BigEndian>()?; // reserved
        let matrix = Matrix::read_from(reader)?;

        let width = FixedPointU32::new_raw(reader.read_u32::<BigEndian>()?);
        let height = FixedPointU32::new_raw(reader.read_u32::<BigEndian>()?);

        skip_bytes_to(reader, start + size)?;

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

impl<W: Write> WriteBox<&mut W> for TkhdBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

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
        writer.write_u16::<BigEndian>(self.volume.raw_value())?;

        writer.write_u16::<BigEndian>(0)?; // reserved

        self.matrix.write_to(writer)?;

        writer.write_u32::<BigEndian>(self.width.raw_value())?;
        writer.write_u32::<BigEndian>(self.height.raw_value())?;

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mp4box::BoxHeader;
    use std::io::Cursor;

    #[test]
    fn test_tkhd32() {
        let src_box = TkhdBox {
            version: 0,
            flags: TrackFlag::TrackEnabled as u32,
            creation_time: 100,
            modification_time: 200,
            track_id: 1,
            duration: 634634,
            layer: 0,
            alternate_group: 0,
            volume: FixedPointU8::new(1),
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
            width: FixedPointU16::new(512),
            height: FixedPointU16::new(288),
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::TkhdBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = TkhdBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }

    #[test]
    fn test_tkhd64() {
        let src_box = TkhdBox {
            version: 1,
            flags: TrackFlag::TrackEnabled as u32,
            creation_time: 100,
            modification_time: 200,
            track_id: 1,
            duration: 634634,
            layer: 0,
            alternate_group: 0,
            volume: FixedPointU8::new(1),
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
            width: FixedPointU16::new(512),
            height: FixedPointU16::new(288),
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::TkhdBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = TkhdBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
