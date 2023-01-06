use crate::mp4box::*;
use crate::Mp4Box;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct VpccBox {
    pub version: u8,
    pub flags: u32,
    pub profile: u8,
    pub level: u8,
    pub bit_depth: u8,
    pub chroma_subsampling: u8,
    pub video_full_range_flag: bool,
    pub color_primaries: u8,
    pub transfer_characteristics: u8,
    pub matrix_coefficients: u8,
    pub codec_initialization_data_size: u16,
}

impl VpccBox {
    pub const DEFAULT_VERSION: u8 = 1;
    pub const DEFAULT_BIT_DEPTH: u8 = 8;
}

impl Mp4Box for VpccBox {
    fn box_type(&self) -> BoxType {
        BoxType::VpccBox
    }

    fn box_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 8
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        Ok(format!("{:?}", self))
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for VpccBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;
        let (version, flags) = read_box_header_ext(reader)?;

        let profile: u8 = reader.read_u8()?;
        let level: u8 = reader.read_u8()?;
        let (bit_depth, chroma_subsampling, video_full_range_flag) = {
            let b = reader.read_u8()?;
            (b >> 4, b << 4 >> 5, b & 0x01 == 1)
        };
        let transfer_characteristics: u8 = reader.read_u8()?;
        let matrix_coefficients: u8 = reader.read_u8()?;
        let codec_initialization_data_size: u16 = reader.read_u16::<BigEndian>()?;

        skip_bytes_to(reader, start + size)?;

        Ok(Self {
            version,
            flags,
            profile,
            level,
            bit_depth,
            chroma_subsampling,
            video_full_range_flag,
            color_primaries: 0,
            transfer_characteristics,
            matrix_coefficients,
            codec_initialization_data_size,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for VpccBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        writer.write_u8(self.profile)?;
        writer.write_u8(self.level)?;
        writer.write_u8(
            (self.bit_depth << 4)
                | (self.chroma_subsampling << 1)
                | (self.video_full_range_flag as u8),
        )?;
        writer.write_u8(self.color_primaries)?;
        writer.write_u8(self.transfer_characteristics)?;
        writer.write_u8(self.matrix_coefficients)?;
        writer.write_u16::<BigEndian>(self.codec_initialization_data_size)?;

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mp4box::BoxHeader;
    use std::io::Cursor;

    #[test]
    fn test_vpcc() {
        let src_box = VpccBox {
            version: VpccBox::DEFAULT_VERSION,
            flags: 0,
            profile: 0,
            level: 0x1F,
            bit_depth: VpccBox::DEFAULT_BIT_DEPTH,
            chroma_subsampling: 0,
            video_full_range_flag: false,
            color_primaries: 0,
            transfer_characteristics: 0,
            matrix_coefficients: 0,
            codec_initialization_data_size: 0,
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::VpccBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = VpccBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
