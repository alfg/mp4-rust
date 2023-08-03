use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Hev1Box {
    pub data_reference_index: u16,
    pub width: u16,
    pub height: u16,

    #[serde(with = "value_u32")]
    pub horizresolution: FixedPointU16,

    #[serde(with = "value_u32")]
    pub vertresolution: FixedPointU16,
    pub frame_count: u16,
    pub depth: u16,
    pub hvcc: HvcCBox,
}

impl Default for Hev1Box {
    fn default() -> Self {
        Hev1Box {
            data_reference_index: 0,
            width: 0,
            height: 0,
            horizresolution: FixedPointU16::new(0x48),
            vertresolution: FixedPointU16::new(0x48),
            frame_count: 1,
            depth: 0x0018,
            hvcc: HvcCBox::default(),
        }
    }
}

impl Hev1Box {
    pub fn new(config: &HevcConfig) -> Self {
        Hev1Box {
            data_reference_index: 1,
            width: config.width,
            height: config.height,
            horizresolution: FixedPointU16::new(0x48),
            vertresolution: FixedPointU16::new(0x48),
            frame_count: 1,
            depth: 0x0018,
            hvcc: HvcCBox::new(),
        }
    }

    pub fn get_type(&self) -> BoxType {
        BoxType::Hev1Box
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE + 8 + 70 + self.hvcc.box_size()
    }
}

impl Mp4Box for Hev1Box {
    fn box_type(&self) -> BoxType {
        self.get_type()
    }

    fn box_size(&self) -> u64 {
        self.get_size()
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!(
            "data_reference_index={} width={} height={} frame_count={}",
            self.data_reference_index, self.width, self.height, self.frame_count
        );
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for Hev1Box {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        reader.read_u32::<BigEndian>()?; // reserved
        reader.read_u16::<BigEndian>()?; // reserved
        let data_reference_index = reader.read_u16::<BigEndian>()?;

        reader.read_u32::<BigEndian>()?; // pre-defined, reserved
        reader.read_u64::<BigEndian>()?; // pre-defined
        reader.read_u32::<BigEndian>()?; // pre-defined
        let width = reader.read_u16::<BigEndian>()?;
        let height = reader.read_u16::<BigEndian>()?;
        let horizresolution = FixedPointU16::new_raw(reader.read_u32::<BigEndian>()?);
        let vertresolution = FixedPointU16::new_raw(reader.read_u32::<BigEndian>()?);
        reader.read_u32::<BigEndian>()?; // reserved
        let frame_count = reader.read_u16::<BigEndian>()?;
        skip_bytes(reader, 32)?; // compressorname
        let depth = reader.read_u16::<BigEndian>()?;
        reader.read_i16::<BigEndian>()?; // pre-defined

        let header = BoxHeader::read(reader)?;
        let BoxHeader { name, size: s } = header;
        if s > size {
            return Err(Error::InvalidData(
                "hev1 box contains a box with a larger size than it",
            ));
        }
        if name == BoxType::HvcCBox {
            let hvcc = HvcCBox::read_box(reader, s)?;

            skip_bytes_to(reader, start + size)?;

            Ok(Hev1Box {
                data_reference_index,
                width,
                height,
                horizresolution,
                vertresolution,
                frame_count,
                depth,
                hvcc,
            })
        } else {
            Err(Error::InvalidData("hvcc not found"))
        }
    }
}

impl<W: Write> WriteBox<&mut W> for Hev1Box {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        writer.write_u32::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(self.data_reference_index)?;

        writer.write_u32::<BigEndian>(0)?; // pre-defined, reserved
        writer.write_u64::<BigEndian>(0)?; // pre-defined
        writer.write_u32::<BigEndian>(0)?; // pre-defined
        writer.write_u16::<BigEndian>(self.width)?;
        writer.write_u16::<BigEndian>(self.height)?;
        writer.write_u32::<BigEndian>(self.horizresolution.raw_value())?;
        writer.write_u32::<BigEndian>(self.vertresolution.raw_value())?;
        writer.write_u32::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(self.frame_count)?;
        // skip compressorname
        write_zeros(writer, 32)?;
        writer.write_u16::<BigEndian>(self.depth)?;
        writer.write_i16::<BigEndian>(-1)?; // pre-defined

        self.hvcc.write_box(writer)?;

        Ok(size)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HvcCBox {
    pub configuration_version: u8,
    pub general_profile_space: u8,
    pub general_tier_flag: bool,
    pub general_profile_idc: u8,
    pub general_profile_compatibility_flags: u32,
    pub general_constraint_indicator_flag: u64,
    pub general_level_idc: u8,
    pub min_spatial_segmentation_idc: u16,
    pub parallelism_type: u8,
    pub chroma_format_idc: u8,
    pub bit_depth_luma_minus8: u8,
    pub bit_depth_chroma_minus8: u8,
    pub avg_frame_rate: u16,
    pub constant_frame_rate: u8,
    pub num_temporal_layers: u8,
    pub temporal_id_nested: bool,
    pub length_size_minus_one: u8,
    pub arrays: Vec<HvcCArray>,
}

impl HvcCBox {
    pub fn new() -> Self {
        Self {
            configuration_version: 1,
            ..Default::default()
        }
    }
}

impl Mp4Box for HvcCBox {
    fn box_type(&self) -> BoxType {
        BoxType::HvcCBox
    }

    fn box_size(&self) -> u64 {
        HEADER_SIZE
            + 23
            + self
                .arrays
                .iter()
                .map(|a| 3 + a.nalus.iter().map(|x| 2 + x.data.len() as u64).sum::<u64>())
                .sum::<u64>()
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        Ok(format!("configuration_version={} general_profile_space={} general_tier_flag={} general_profile_idc={} general_profile_compatibility_flags={} general_constraint_indicator_flag={} general_level_idc={} min_spatial_segmentation_idc={} parallelism_type={} chroma_format_idc={} bit_depth_luma_minus8={} bit_depth_chroma_minus8={} avg_frame_rate={} constant_frame_rate={} num_temporal_layers={} temporal_id_nested={} length_size_minus_one={}", 
            self.configuration_version,
            self.general_profile_space,
            self.general_tier_flag,
            self.general_profile_idc,
            self.general_profile_compatibility_flags,
            self.general_constraint_indicator_flag,
            self.general_level_idc,
            self.min_spatial_segmentation_idc,
            self.parallelism_type,
            self.chroma_format_idc,
            self.bit_depth_luma_minus8,
            self.bit_depth_chroma_minus8,
            self.avg_frame_rate,
            self.constant_frame_rate,
            self.num_temporal_layers,
            self.temporal_id_nested,
            self.length_size_minus_one
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct HvcCArrayNalu {
    pub size: u16,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct HvcCArray {
    pub completeness: bool,
    pub nal_unit_type: u8,
    pub nalus: Vec<HvcCArrayNalu>,
}

impl<R: Read + Seek> ReadBox<&mut R> for HvcCBox {
    fn read_box(reader: &mut R, _size: u64) -> Result<Self> {
        let configuration_version = reader.read_u8()?;
        let params = reader.read_u8()?;
        let general_profile_space = params & 0b11000000 >> 6;
        let general_tier_flag = (params & 0b00100000 >> 5) > 0;
        let general_profile_idc = params & 0b00011111;

        let general_profile_compatibility_flags = reader.read_u32::<BigEndian>()?;
        let general_constraint_indicator_flag = reader.read_u48::<BigEndian>()?;
        let general_level_idc = reader.read_u8()?;
        let min_spatial_segmentation_idc = reader.read_u16::<BigEndian>()? & 0x0FFF;
        let parallelism_type = reader.read_u8()? & 0b11;
        let chroma_format_idc = reader.read_u8()? & 0b11;
        let bit_depth_luma_minus8 = reader.read_u8()? & 0b111;
        let bit_depth_chroma_minus8 = reader.read_u8()? & 0b111;
        let avg_frame_rate = reader.read_u16::<BigEndian>()?;

        let params = reader.read_u8()?;
        let constant_frame_rate = params & 0b11000000 >> 6;
        let num_temporal_layers = params & 0b00111000 >> 3;
        let temporal_id_nested = (params & 0b00000100 >> 2) > 0;
        let length_size_minus_one = params & 0b000011;

        let num_of_arrays = reader.read_u8()?;

        let mut arrays = Vec::with_capacity(num_of_arrays as _);
        for _ in 0..num_of_arrays {
            let params = reader.read_u8()?;
            let num_nalus = reader.read_u16::<BigEndian>()?;
            let mut nalus = Vec::with_capacity(num_nalus as usize);

            for _ in 0..num_nalus {
                let size = reader.read_u16::<BigEndian>()?;
                let mut data = vec![0; size as usize];

                reader.read_exact(&mut data)?;

                nalus.push(HvcCArrayNalu { size, data })
            }

            arrays.push(HvcCArray {
                completeness: (params & 0b10000000) > 0,
                nal_unit_type: params & 0b111111,
                nalus,
            });
        }

        Ok(HvcCBox {
            configuration_version,
            general_profile_space,
            general_tier_flag,
            general_profile_idc,
            general_profile_compatibility_flags,
            general_constraint_indicator_flag,
            general_level_idc,
            min_spatial_segmentation_idc,
            parallelism_type,
            chroma_format_idc,
            bit_depth_luma_minus8,
            bit_depth_chroma_minus8,
            avg_frame_rate,
            constant_frame_rate,
            num_temporal_layers,
            temporal_id_nested,
            length_size_minus_one,
            arrays,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for HvcCBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        writer.write_u8(self.configuration_version)?;
        let general_profile_space = (self.general_profile_space & 0b11) << 6;
        let general_tier_flag = u8::from(self.general_tier_flag) << 5;
        let general_profile_idc = self.general_profile_idc & 0b11111;

        writer.write_u8(general_profile_space | general_tier_flag | general_profile_idc)?;
        writer.write_u32::<BigEndian>(self.general_profile_compatibility_flags)?;
        writer.write_u48::<BigEndian>(self.general_constraint_indicator_flag)?;
        writer.write_u8(self.general_level_idc)?;

        writer.write_u16::<BigEndian>(self.min_spatial_segmentation_idc & 0x0FFF)?;
        writer.write_u8(self.parallelism_type & 0b11)?;
        writer.write_u8(self.chroma_format_idc & 0b11)?;
        writer.write_u8(self.bit_depth_luma_minus8 & 0b111)?;
        writer.write_u8(self.bit_depth_chroma_minus8 & 0b111)?;
        writer.write_u16::<BigEndian>(self.avg_frame_rate)?;

        let constant_frame_rate = (self.constant_frame_rate & 0b11) << 6;
        let num_temporal_layers = (self.num_temporal_layers & 0b111) << 3;
        let temporal_id_nested = u8::from(self.temporal_id_nested) << 2;
        let length_size_minus_one = self.length_size_minus_one & 0b11;
        writer.write_u8(
            constant_frame_rate | num_temporal_layers | temporal_id_nested | length_size_minus_one,
        )?;
        writer.write_u8(self.arrays.len() as u8)?;
        for arr in &self.arrays {
            writer.write_u8((arr.nal_unit_type & 0b111111) | u8::from(arr.completeness) << 7)?;
            writer.write_u16::<BigEndian>(arr.nalus.len() as _)?;

            for nalu in &arr.nalus {
                writer.write_u16::<BigEndian>(nalu.size)?;
                writer.write_all(&nalu.data)?;
            }
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
    fn test_hev1() {
        let src_box = Hev1Box {
            data_reference_index: 1,
            width: 320,
            height: 240,
            horizresolution: FixedPointU16::new(0x48),
            vertresolution: FixedPointU16::new(0x48),
            frame_count: 1,
            depth: 24,
            hvcc: HvcCBox {
                configuration_version: 1,
                ..Default::default()
            },
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::Hev1Box);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = Hev1Box::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
