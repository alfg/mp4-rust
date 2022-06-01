use crate::mp4box::vpcc::VpccBox;
use crate::mp4box::*;
use crate::Mp4Box;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct Vp09Box {
    pub version: u8,
    pub flags: u32,
    pub start_code: u16,
    pub data_reference_index: u16,
    pub reserved0: [u8; 16],
    pub width: u16,
    pub height: u16,
    pub horizresolution: (u16, u16),
    pub vertresolution: (u16, u16),
    pub reserved1: [u8; 4],
    pub frame_count: u16,
    pub compressorname: [u8; 32],
    pub depth: u16,
    pub end_code: u16,
    pub vpcc: VpccBox,
}

impl Vp09Box {
    pub const DEFAULT_START_CODE: u16 = 0;
    pub const DEFAULT_END_CODE: u16 = 0xFFFF;
    pub const DEFAULT_DATA_REFERENCE_INDEX: u16 = 1;
    pub const DEFAULT_HORIZRESOLUTION: (u16, u16) = (0x48, 0x00);
    pub const DEFAULT_VERTRESOLUTION: (u16, u16) = (0x48, 0x00);
    pub const DEFAULT_FRAME_COUNT: u16 = 1;
    pub const DEFAULT_COMPRESSORNAME: [u8; 32] = [0; 32];
    pub const DEFAULT_DEPTH: u16 = 24;

    pub fn new(config: &Vp9Config) -> Self {
        Vp09Box {
            version: 0,
            flags: 0,
            start_code: Vp09Box::DEFAULT_START_CODE,
            data_reference_index: Vp09Box::DEFAULT_DATA_REFERENCE_INDEX,
            reserved0: Default::default(),
            width: config.width,
            height: config.height,
            horizresolution: Vp09Box::DEFAULT_HORIZRESOLUTION,
            vertresolution: Vp09Box::DEFAULT_VERTRESOLUTION,
            reserved1: Default::default(),
            frame_count: Vp09Box::DEFAULT_FRAME_COUNT,
            compressorname: Vp09Box::DEFAULT_COMPRESSORNAME,
            depth: Vp09Box::DEFAULT_DEPTH,
            end_code: Vp09Box::DEFAULT_END_CODE,
            vpcc: VpccBox {
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
            },
        }
    }
}

impl Mp4Box for Vp09Box {
    fn box_type(&self) -> BoxType {
        BoxType::Vp09Box
    }

    fn box_size(&self) -> u64 {
        0x6A
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        Ok(format!("{:?}", self))
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for Vp09Box {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;
        let (version, flags) = read_box_header_ext(reader)?;

        let start_code: u16 = reader.read_u16::<BigEndian>()?;
        let data_reference_index: u16 = reader.read_u16::<BigEndian>()?;
        let reserved0: [u8; 16] = {
            let mut buf = [0u8; 16];
            reader.read_exact(&mut buf)?;
            buf
        };
        let width: u16 = reader.read_u16::<BigEndian>()?;
        let height: u16 = reader.read_u16::<BigEndian>()?;
        let horizresolution: (u16, u16) = (
            reader.read_u16::<BigEndian>()?,
            reader.read_u16::<BigEndian>()?,
        );
        let vertresolution: (u16, u16) = (
            reader.read_u16::<BigEndian>()?,
            reader.read_u16::<BigEndian>()?,
        );
        let reserved1: [u8; 4] = {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf)?;
            buf
        };
        let frame_count: u16 = reader.read_u16::<BigEndian>()?;
        let compressorname: [u8; 32] = {
            let mut buf = [0u8; 32];
            reader.read_exact(&mut buf)?;
            buf
        };
        let depth: u16 = reader.read_u16::<BigEndian>()?;
        let end_code: u16 = reader.read_u16::<BigEndian>()?;

        let vpcc = {
            let header = BoxHeader::read(reader)?;
            VpccBox::read_box(reader, header.size)?
        };

        skip_bytes_to(reader, start + size)?;

        Ok(Self {
            version,
            flags,
            start_code,
            data_reference_index,
            reserved0,
            width,
            height,
            horizresolution,
            vertresolution,
            reserved1,
            frame_count,
            compressorname,
            depth,
            end_code,
            vpcc,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for Vp09Box {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        writer.write_u16::<BigEndian>(self.start_code)?;
        writer.write_u16::<BigEndian>(self.data_reference_index)?;
        writer.write_all(&self.reserved0)?;
        writer.write_u16::<BigEndian>(self.width)?;
        writer.write_u16::<BigEndian>(self.height)?;
        writer.write_u16::<BigEndian>(self.horizresolution.0)?;
        writer.write_u16::<BigEndian>(self.horizresolution.1)?;
        writer.write_u16::<BigEndian>(self.vertresolution.0)?;
        writer.write_u16::<BigEndian>(self.vertresolution.1)?;
        writer.write_all(&self.reserved1)?;
        writer.write_u16::<BigEndian>(self.frame_count)?;
        writer.write_all(&self.compressorname)?;
        writer.write_u16::<BigEndian>(self.depth)?;
        writer.write_u16::<BigEndian>(self.end_code)?;
        VpccBox::write_box(&self.vpcc, writer)?;

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
        let src_box = Vp09Box::new(&Vp9Config {
            width: 1920,
            height: 1080,
        });
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::Vp09Box);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = Vp09Box::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
