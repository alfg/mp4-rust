use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct SencBox {
    pub version: u8,
    pub flags: u32,
    pub sample_count: u32,
    pub sample_data: Vec<u8>,
}

impl SencBox {
    pub const FLAG_USE_SUBSAMPLE_ENCRYPTION: u32 = 0x02;

    pub fn get_type(&self) -> BoxType {
        BoxType::SencBox
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 4 + (self.sample_data.len() as u64)
    }

    pub fn get_sample_info(&self, iv_size: u8) -> Result<Vec<SampleInfo>> {
        if iv_size != 16 && iv_size != 8 && iv_size != 0 {
            return Err(Error::InvalidData("invalid iv_size"));
        }
        let mut reader = &self.sample_data[..];
        let mut infos = Vec::with_capacity(self.sample_count as usize);
        for _ in 0..self.sample_count {
            let mut iv = vec![0; iv_size as usize];
            if iv_size != 0 {
                reader.read_exact(&mut iv)?;
            }
            let mut subsamples = Vec::new();
            if SencBox::FLAG_USE_SUBSAMPLE_ENCRYPTION & self.flags != 0 {
                let subsample_count = reader.read_u16::<BigEndian>()?;
                subsamples = Vec::with_capacity(subsample_count as usize);
                for _ in 0..subsample_count {
                    let bytes_of_clear_data = reader.read_u16::<BigEndian>()?;
                    let bytes_of_encrypted_data = reader.read_u32::<BigEndian>()?;
                    subsamples.push(SubSampleInfo {
                        bytes_of_clear_data,
                        bytes_of_encrypted_data,
                    });
                }
            }
            infos.push(SampleInfo { iv, subsamples });
        }
        Ok(infos)
    }

    pub fn set_sample_info(&mut self, infos: &[SampleInfo], iv_size: u8) -> Result<()> {
        if iv_size != 16 && iv_size != 8 && iv_size != 0 {
            return Err(Error::InvalidData("invalid iv_size"));
        }
        let mut buf = Vec::new();
        for info in infos {
            if iv_size != 0 {
                buf.write_all(&info.iv[..iv_size as usize])?;
            }
            if SencBox::FLAG_USE_SUBSAMPLE_ENCRYPTION & self.flags != 0 {
                buf.write_u16::<BigEndian>(info.subsamples.len() as u16)?;
                for subsample in &info.subsamples {
                    buf.write_u16::<BigEndian>(subsample.bytes_of_clear_data)?;
                    buf.write_u32::<BigEndian>(subsample.bytes_of_encrypted_data)?;
                }
            }
        }
        self.sample_data = buf;
        Ok(())
    }
}

impl Mp4Box for SencBox {
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
        let s = format!("sample_count={}", self.sample_count);
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for SencBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        let sample_count = reader.read_u32::<BigEndian>()?;

        // the senc box cannot be properly parsed without IV_size
        // which is only available from other boxes. Store the raw
        // data for parsing with member functions later
        let data_size = start + size - reader.stream_position()?;
        let mut sample_data = vec![0; data_size as usize];
        reader.read_exact(&mut sample_data)?;

        skip_bytes_to(reader, start + size)?;

        Ok(SencBox {
            version,
            flags,
            sample_count,
            sample_data,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for SencBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        writer.write_u32::<BigEndian>(self.sample_count)?;

        writer.write_all(&self.sample_data)?;

        Ok(size)
    }
}
