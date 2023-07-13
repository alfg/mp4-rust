use crate::mp4box::*;
use crate::Mp4Box;
use serde::Serialize;

// taken from the following sources
// - https://opus-codec.org/docs/opus_in_isobmff.html
// - chromium source code: box_definitions.h - OpusSpecificBox
// - async-mp4 crate: https://github.com/Wicpar/async-mp4/blob/master/src/mp4box/dops.rs

// this OpusBox is a combination of the AudioSampleEntry box and OpusSpecificBox
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OpusBox {
    pub data_reference_index: u16,
    pub channelcount: u16,
    pub samplesize: u16,

    #[serde(with = "value_u32")]
    pub samplerate: FixedPointU16,
    pub dops: DopsBox,
}

impl Mp4Box for OpusBox {
    fn box_type(&self) -> BoxType {
        BoxType::OpusBox
    }

    fn box_size(&self) -> u64 {
        // the +19 is for DopsBox
        36 + 19
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        Ok(format!("{self:?}"))
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for OpusBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        box_start(reader)?;
        reader.read_u32::<BigEndian>()?; // reserved
        reader.read_u16::<BigEndian>()?; // reserved
        let data_reference_index = reader.read_u16::<BigEndian>()?;
        reader.read_u64::<BigEndian>()?; // reserved
        let channelcount = reader.read_u16::<BigEndian>()?;
        let samplesize = reader.read_u16::<BigEndian>()?;
        reader.read_u32::<BigEndian>()?; // reserved
        let samplerate = reader.read_u32::<BigEndian>()?;

        let num_read = 36;
        if num_read > size {
            return Err(Error::InvalidData("opus box is too small"));
        }
        let remaining = num_read.saturating_sub(size);

        // todo: if the size of DopsBox becomes variable, then the size of
        // OpusBox will be needed to determine what size to pass to DopsBox::read_box()
        let dops = DopsBox::read_box(reader, remaining)?;

        Ok(Self {
            data_reference_index,
            channelcount,
            samplesize,
            samplerate: FixedPointU16::new_raw(samplerate),
            dops,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for OpusBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let mut written = 0;
        written += BoxHeader::new(self.box_type(), self.box_size()).write(writer)?;

        writer.write_u32::<BigEndian>(0)?; // reserved
        written += 4;
        writer.write_u16::<BigEndian>(0)?; // reserved
        written += 2;
        writer.write_u16::<BigEndian>(self.data_reference_index)?;
        written += 2;

        writer.write_u64::<BigEndian>(0)?; // reserved
        written += 8;
        writer.write_u16::<BigEndian>(self.channelcount)?;
        written += 2;
        writer.write_u16::<BigEndian>(self.samplesize)?;
        written += 2;
        writer.write_u32::<BigEndian>(0)?; // reserved
        written += 4;
        writer.write_u32::<BigEndian>(self.samplerate.raw_value())?;
        written += 4;

        written += self.dops.write_box(writer)?;

        assert_eq!(written, self.box_size());
        Ok(written)
    }
}

// https://github.com/Wicpar/async-mp4/blob/master/src/mp4box/dops.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DopsBox {
    pub version: u8,
    pub pre_skip: u16,
    pub input_sample_rate: u32,
    pub output_gain: i16,
    pub channel_mapping_family: ChannelMappingFamily,
}

impl Mp4Box for DopsBox {
    fn box_type(&self) -> BoxType {
        BoxType::DopsBox
    }

    fn box_size(&self) -> u64 {
        // if channel_mapping_family is updates to support more than 2 channels,
        // box_size could change, depending on the channel mapping.
        19
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        Ok(format!("{self:?}"))
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for DopsBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        if size < 19 {
            return Err(Error::InvalidData("Dops box size too small"));
        }
        box_start(reader)?;
        let version = reader.read_u8()?;
        let num_channels = reader.read_u8()?;
        let pre_skip = reader.read_u16::<BigEndian>()?;
        let input_sample_rate = reader.read_u32::<BigEndian>()?;
        let output_gain = reader.read_i16::<BigEndian>()?;

        // todo: should this be used?
        let _channel_mapping_family = reader.read_u8()?;

        Ok(DopsBox {
            version,
            pre_skip,
            input_sample_rate,
            output_gain,
            channel_mapping_family: ChannelMappingFamily::Family0 {
                stereo: num_channels == 2,
            },
        })
    }
}

// https://github.com/Wicpar/async-mp4/blob/master/src/mp4box/dops.rs
impl<W: Write> WriteBox<&mut W> for DopsBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let mut written = 0;
        written += BoxHeader::new(self.box_type(), self.box_size()).write(writer)?;
        writer.write_u8(self.version)?;
        written += 1;

        let num_channels = match self.channel_mapping_family {
            ChannelMappingFamily::Family0 { stereo } => match stereo {
                true => 2,
                false => 1,
            },
        };
        writer.write_u8(num_channels)?;
        written += 1;
        writer.write_u16::<BigEndian>(self.pre_skip)?;
        written += 2;
        writer.write_u32::<BigEndian>(self.input_sample_rate)?;
        written += 4;
        writer.write_i16::<BigEndian>(self.output_gain)?;
        written += 2;

        // channel mapping family 0
        writer.write_u8(0)?;
        written += 1;

        // todo: StreamCount? CoupledCount? ChannelMapping?

        assert_eq!(written, self.box_size());
        Ok(written)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ChannelMappingFamily {
    Family0 { stereo: bool },
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_opus_writer() {
        let dops = DopsBox {
            version: 0,
            pre_skip: 1,
            input_sample_rate: 2,
            output_gain: 3,
            channel_mapping_family: ChannelMappingFamily::Family0 { stereo: false },
        };

        let opus = OpusBox {
            data_reference_index: 1,
            channelcount: 1,
            samplesize: 2,
            samplerate: FixedPointU16::new(48000),
            dops,
        };

        let mut buffer = Vec::<u8>::new();
        opus.write_box(&mut buffer).expect("write_box failed");
    }
}
