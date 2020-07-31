use std::io::{Seek, Read, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use num_rational::Ratio;

use crate::*;
use crate::atoms::*;


#[derive(Debug, PartialEq)]
pub struct Avc1Box {
    pub data_reference_index: u16,
    pub width: u16,
    pub height: u16,
    pub horizresolution: Ratio<u32>,
    pub vertresolution: Ratio<u32>,
    pub frame_count: u16,
    pub depth: u16,
    pub avcc: AvcCBox,
}

impl Default for Avc1Box {
    fn default() -> Self {
        Avc1Box {
            data_reference_index: 0,
            width: 0,
            height: 0,
            horizresolution: Ratio::new_raw(0x00480000, 0x10000),
            vertresolution: Ratio::new_raw(0x00480000, 0x10000),
            frame_count: 1,
            depth: 0x0018,
            avcc: AvcCBox::default(),
        }
    }
}

impl Mp4Box for Avc1Box {
    fn box_type() -> BoxType {
        BoxType::Avc1Box
    }

    fn box_size(&self) -> u64 {
        HEADER_SIZE + 8 + 74 + self.avcc.box_size()
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for Avc1Box {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = get_box_start(reader)?;

        reader.read_u32::<BigEndian>()?; // reserved
        reader.read_u16::<BigEndian>()?; // reserved
        let data_reference_index = reader.read_u16::<BigEndian>()?;

        reader.read_u32::<BigEndian>()?; // pre-defined, reserved
        reader.read_u64::<BigEndian>()?; // pre-defined
        reader.read_u32::<BigEndian>()?; // pre-defined
        let width = reader.read_u16::<BigEndian>()?;
        let height = reader.read_u16::<BigEndian>()?;
        let horiznumer = reader.read_u32::<BigEndian>()?;
        let horizresolution = Ratio::new_raw(horiznumer, 0x10000);
        let vertnumer = reader.read_u32::<BigEndian>()?;
        let vertresolution = Ratio::new_raw(vertnumer, 0x10000);
        reader.read_u32::<BigEndian>()?; // reserved
        let frame_count = reader.read_u16::<BigEndian>()?;
        skip_read(reader, 32)?; // compressorname
        let depth = reader.read_u16::<BigEndian>()?;
        reader.read_i16::<BigEndian>()?; // pre-defined

        let header = BoxHeader::read(reader)?;
        let BoxHeader{ name, size: s } = header;
        if name == BoxType::AvcCBox {
            let avcc = AvcCBox::read_box(reader, s)?;

            skip_read_to(reader, start + size)?;

            Ok(Avc1Box {
                data_reference_index,
                width,
                height,
                horizresolution,
                vertresolution,
                frame_count,
                depth,
                avcc,
            })
        } else {
            Err(Error::InvalidData("avcc not found"))
        }
    }
}

impl<W: Write> WriteBox<&mut W> for Avc1Box {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(Self::box_type(), size).write(writer)?;

        writer.write_u32::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(self.data_reference_index)?;

        writer.write_u32::<BigEndian>(0)?; // pre-defined, reserved
        writer.write_u64::<BigEndian>(0)?; // pre-defined
        writer.write_u32::<BigEndian>(0)?; // pre-defined
        writer.write_u16::<BigEndian>(self.width)?;
        writer.write_u16::<BigEndian>(self.height)?;
        writer.write_u32::<BigEndian>(*self.horizresolution.numer())?;
        writer.write_u32::<BigEndian>(*self.vertresolution.numer())?;
        writer.write_u32::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(self.frame_count)?;
        // skip compressorname
        for _ in 0..4 {
            writer.write_u64::<BigEndian>(0)?;
        }
        writer.write_u16::<BigEndian>(self.depth)?;
        writer.write_i16::<BigEndian>(-1)?; // pre-defined

        self.avcc.write_box(writer)?;

        Ok(size)
    }
}


#[derive(Debug, Default, PartialEq)]
pub struct AvcCBox {
    pub configuration_version: u8,
    pub avc_profile_indication: u8,
    pub profile_compatibility: u8,
    pub avc_level_indication: u8,
    pub length_size_minus_one: u8,
    pub sequence_parameter_sets: Vec<NalUnit>,
    pub picture_parameter_sets: Vec<NalUnit>,
}

impl Mp4Box for AvcCBox {
    fn box_type() -> BoxType {
        BoxType::AvcCBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE + 7;
        for sps in self.sequence_parameter_sets.iter() {
            size += sps.size() as u64;
        }
        for pps in self.picture_parameter_sets.iter() {
            size += pps.size() as u64;
        }
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for AvcCBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = get_box_start(reader)?;

        let configuration_version = reader.read_u8()?;
        let avc_profile_indication = reader.read_u8()?;
        let profile_compatibility = reader.read_u8()?;
        let avc_level_indication = reader.read_u8()?;
        let length_size_minus_one = reader.read_u8()? & 0x3;
        let num_of_spss = reader.read_u8()? & 0x1F;
        let mut sequence_parameter_sets = Vec::with_capacity(num_of_spss as usize);
        for _ in 0..num_of_spss {
            let nal_unit = NalUnit::read(reader)?;
            sequence_parameter_sets.push(nal_unit);
        }
        let num_of_ppss = reader.read_u8()?;
        let mut picture_parameter_sets = Vec::with_capacity(num_of_ppss as usize);
        for _ in 0..num_of_ppss {
            let nal_unit = NalUnit::read(reader)?;
            picture_parameter_sets.push(nal_unit);
        }

        skip_read_to(reader, start + size)?;

        Ok(AvcCBox {
            configuration_version,
            avc_profile_indication,
            profile_compatibility,
            avc_level_indication,
            length_size_minus_one,
            sequence_parameter_sets,
            picture_parameter_sets,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for AvcCBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(Self::box_type(), size).write(writer)?;

        writer.write_u8(self.configuration_version)?;
        writer.write_u8(self.avc_profile_indication)?;
        writer.write_u8(self.profile_compatibility)?;
        writer.write_u8(self.avc_level_indication)?;
        writer.write_u8(self.length_size_minus_one | 0xFC)?;
        writer.write_u8(self.sequence_parameter_sets.len() as u8 | 0xE0)?;
        for sps in self.sequence_parameter_sets.iter() {
            sps.write(writer)?;
        }
        writer.write_u8(self.picture_parameter_sets.len() as u8)?;
        for pps in self.picture_parameter_sets.iter() {
            pps.write(writer)?;
        }
        Ok(size)
    }
}


#[derive(Debug, Default, PartialEq)]
pub struct NalUnit {
    pub bytes: Vec<u8>,
}

impl NalUnit {
    pub fn size(&self) -> usize {
        2 + self.bytes.len()
    }

    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let length = reader.read_u16::<BigEndian>()? as usize;
        let mut bytes = vec![0u8; length];
        reader.read(&mut bytes)?;
        Ok(NalUnit {
            bytes,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<u64> {
        writer.write_u16::<BigEndian>(self.bytes.len() as u16)?;
        writer.write(&self.bytes)?;
        Ok(self.size() as u64)
    }
}
