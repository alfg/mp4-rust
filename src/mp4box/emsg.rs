use std::ffi::CStr;
use std::io::{Read, Seek, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct EmsgBox {
    pub version: u8,
    pub flags: u32,
    pub timescale: u32,
    pub presentation_time: Option<u64>,
    pub presentation_time_delta: Option<u32>,
    pub event_duration: u32,
    pub id: u32,
    pub scheme_id_uri: String,
    pub value: String,
    pub message_data: Vec<u8>,
}

impl EmsgBox {
    fn size_without_message(version: u8, scheme_id_uri: &str, value: &str) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE +
            4 + // id
            Self::time_size(version) +
            (scheme_id_uri.len() + 1) as u64 +
            (value.len() as u64 + 1)
    }

    fn time_size(version: u8) -> u64 {
        match version {
            0 => 12,
            1 => 16,
            _ => panic!("version must be 0 or 1"),
        }
    }
}

impl Mp4Box for EmsgBox {
    fn box_type(&self) -> BoxType {
        BoxType::EmsgBox
    }

    fn box_size(&self) -> u64 {
        Self::size_without_message(self.version, &self.scheme_id_uri, &self.value)
            + self.message_data.len() as u64
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("id={} value={}", self.id, self.value);
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for EmsgBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;
        let (version, flags) = read_box_header_ext(reader)?;

        let (
            timescale,
            presentation_time,
            presentation_time_delta,
            event_duration,
            id,
            scheme_id_uri,
            value,
        ) = match version {
            0 => {
                let scheme_id_uri = read_null_terminated_utf8_string(reader)?;
                let value = read_null_terminated_utf8_string(reader)?;
                (
                    reader.read_u32::<BigEndian>()?,
                    None,
                    Some(reader.read_u32::<BigEndian>()?),
                    reader.read_u32::<BigEndian>()?,
                    reader.read_u32::<BigEndian>()?,
                    scheme_id_uri,
                    value,
                )
            }
            1 => (
                reader.read_u32::<BigEndian>()?,
                Some(reader.read_u64::<BigEndian>()?),
                None,
                reader.read_u32::<BigEndian>()?,
                reader.read_u32::<BigEndian>()?,
                read_null_terminated_utf8_string(reader)?,
                read_null_terminated_utf8_string(reader)?,
            ),
            _ => return Err(Error::InvalidData("version must be 0 or 1")),
        };

        let message_size = size - Self::size_without_message(version, &scheme_id_uri, &value);
        let mut message_data = Vec::with_capacity(message_size as usize);
        for _ in 0..message_size {
            message_data.push(reader.read_u8()?);
        }

        skip_bytes_to(reader, start + size)?;

        Ok(EmsgBox {
            version,
            flags,
            timescale,
            presentation_time,
            presentation_time_delta,
            event_duration,
            id,
            scheme_id_uri,
            value,
            message_data,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for EmsgBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;
        match self.version {
            0 => {
                write_null_terminated_str(writer, &self.scheme_id_uri)?;
                write_null_terminated_str(writer, &self.value)?;
                writer.write_u32::<BigEndian>(self.timescale)?;
                writer.write_u32::<BigEndian>(self.presentation_time_delta.unwrap())?;
                writer.write_u32::<BigEndian>(self.event_duration)?;
                writer.write_u32::<BigEndian>(self.id)?;
            }
            1 => {
                writer.write_u32::<BigEndian>(self.timescale)?;
                writer.write_u64::<BigEndian>(self.presentation_time.unwrap())?;
                writer.write_u32::<BigEndian>(self.event_duration)?;
                writer.write_u32::<BigEndian>(self.id)?;
                write_null_terminated_str(writer, &self.scheme_id_uri)?;
                write_null_terminated_str(writer, &self.value)?;
            }
            _ => return Err(Error::InvalidData("version must be 0 or 1")),
        }

        for &byte in &self.message_data {
            writer.write_u8(byte)?;
        }

        Ok(size)
    }
}

fn read_null_terminated_utf8_string<R: Read + Seek>(reader: &mut R) -> Result<String> {
    let mut bytes = Vec::new();
    loop {
        let byte = reader.read_u8()?;
        bytes.push(byte);
        if byte == 0 {
            break;
        }
    }
    if let Ok(str) = unsafe { CStr::from_bytes_with_nul_unchecked(&bytes) }.to_str() {
        Ok(str.to_string())
    } else {
        Err(Error::InvalidData("invalid utf8"))
    }
}

fn write_null_terminated_str<W: Write>(writer: &mut W, string: &str) -> Result<()> {
    for byte in string.bytes() {
        writer.write_u8(byte)?;
    }
    writer.write_u8(0)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::mp4box::BoxHeader;

    use super::*;

    #[test]
    fn test_emsg_version0() {
        let src_box = EmsgBox {
            version: 0,
            flags: 0,
            timescale: 48000,
            presentation_time: None,
            presentation_time_delta: Some(100),
            event_duration: 200,
            id: 8,
            scheme_id_uri: String::from("foo"),
            value: String::from("foo"),
            message_data: vec![1, 2, 3],
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::EmsgBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = EmsgBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }

    #[test]
    fn test_emsg_version1() {
        let src_box = EmsgBox {
            version: 1,
            flags: 0,
            timescale: 48000,
            presentation_time: Some(50000),
            presentation_time_delta: None,
            event_duration: 200,
            id: 8,
            scheme_id_uri: String::from("foo"),
            value: String::from("foo"),
            message_data: vec![3, 2, 1],
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::EmsgBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = EmsgBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
