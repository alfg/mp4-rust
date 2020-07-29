use std::fmt;
use std::io::{BufReader, SeekFrom, Seek, Read, BufWriter, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::*;

mod ftyp;
mod moov;
mod mvhd;
mod trak;
mod tkhd;
mod edts;
mod elst;
mod mdia;
mod mdhd;
mod hdlr;
mod minf;
mod vmhd;
mod stbl;
mod stts;
mod stsd;

pub use ftyp::FtypBox;
pub use moov::MoovBox;

pub const HEADER_EXT_SIZE: u64 = 4;

macro_rules! boxtype {
    ($( $name:ident => $value:expr ),*) => {
        #[derive(Clone, Copy, PartialEq)]
        pub enum BoxType {
            $( $name, )*
            UnknownBox(u32),
        }

        impl From<u32> for BoxType {
            fn from(t: u32) -> BoxType {
                match t {
                    $( $value => BoxType::$name, )*
                    _ => BoxType::UnknownBox(t),
                }
            }
        }

        impl Into<u32> for BoxType {
            fn into(self) -> u32 {
                match self {
                    $( BoxType::$name => $value, )*
                    BoxType::UnknownBox(t) => t,
                }
            }
        }
    }
}

boxtype!{
    FtypBox => 0x66747970,
    MvhdBox => 0x6d766864,
    FreeBox => 0x66726565,
    MdatBox => 0x6d646174,
    MoovBox => 0x6d6f6f76,
    MoofBox => 0x6d6f6f66,
    TkhdBox => 0x746b6864,
    EdtsBox => 0x65647473,
    MdiaBox => 0x6d646961,
    ElstBox => 0x656c7374,
    MdhdBox => 0x6d646864,
    HdlrBox => 0x68646c72,
    MinfBox => 0x6d696e66,
    VmhdBox => 0x766d6864,
    StblBox => 0x7374626c,
    StsdBox => 0x73747364,
    SttsBox => 0x73747473,
    TrakBox => 0x7472616b,
    UdtaBox => 0x75647461,
    DinfBox => 0x64696e66,
    SmhdBox => 0x736d6864,
    Avc1Box => 0x61766331,
    Mp4aBox => 0x6d703461
}

impl fmt::Debug for BoxType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fourcc: FourCC = From::from(self.clone());
        write!(f, "{}", fourcc)
    }
}

#[derive(Default, PartialEq, Clone)]
pub struct FourCC {
    pub value: String
}

impl From<u32> for FourCC {
    fn from(number: u32) -> Self {
        let mut box_chars = Vec::new();
        for x in 0..4 {
            let c = (number >> (x * 8) & 0x0000_00FF) as u8;
            box_chars.push(c);
        }
        box_chars.reverse();

        let box_string = match String::from_utf8(box_chars) {
            Ok(t) => t,
            _ => String::from("null"), // error to retrieve fourcc
        };

        FourCC {
            value: box_string
        }
    }
}

impl From<FourCC> for u32 {
    fn from(fourcc: FourCC) -> u32 {
        (&fourcc).into()
    }
}

impl From<&FourCC> for u32 {
    fn from(fourcc: &FourCC) -> u32 {
        let mut b: [u8; 4] = Default::default();
        b.copy_from_slice(fourcc.value.as_bytes());
        u32::from_be_bytes(b)
    }
}

impl From<String> for FourCC {
    fn from(fourcc: String) -> FourCC {
        let value = if fourcc.len() > 4 {
            fourcc[0..4].to_string()
        } else {
            fourcc
        };
        FourCC {value}
    }
}

impl From<&str> for FourCC {
    fn from(fourcc: &str) -> FourCC {
        let value = if fourcc.len() > 4 {
            fourcc[0..4].to_string()
        } else {
            fourcc.to_string()
        };
        FourCC {value}
    }
}

impl From<BoxType> for FourCC {
    fn from(t: BoxType) -> FourCC {
        let box_num: u32 = Into::into(t);
        From::from(box_num)
    }
}

impl fmt::Debug for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Display for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

pub trait Mp4Box: Sized {
    fn box_type(&self) -> BoxType;
    fn box_size(&self) -> u64;
}

pub trait ReadBox<T>: Sized {
    fn read_box(_: T, size: u64) -> Result<Self>;
}

pub trait WriteBox<T>: Sized {
    fn write_box(&self, _: T) -> Result<u64>;
}

pub fn read_box_header_ext<R: Read>(reader: &mut BufReader<R>) -> Result<(u8, u32)> {
    let version = reader.read_u8()?;
    let flags_a = reader.read_u8()?;
    let flags_b = reader.read_u8()?;
    let flags_c = reader.read_u8()?;
    let flags = u32::from(flags_a) << 16 | u32::from(flags_b) << 8 | u32::from(flags_c);
    Ok((version, flags))
}

pub fn write_box_header_ext<W: Write>(w: &mut BufWriter<W>, v: u8, f: u32) -> Result<u64> {
    let d = u32::from(v) << 24 | f;
    w.write_u32::<BigEndian>(d)?;
    Ok(4)
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for BoxHeader {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        if self.size > u32::MAX as u64 {
            writer.write_u32::<BigEndian>(1)?;
            writer.write_u32::<BigEndian>(self.name.into())?;
            writer.write_u64::<BigEndian>(self.size)?;
            Ok(16)
        } else {
            writer.write_u32::<BigEndian>(self.size as u32)?;
            writer.write_u32::<BigEndian>(self.name.into())?;
            Ok(8)
        }
    }
}

pub fn skip_read<R: Read + Seek>(reader: &mut BufReader<R>, current: u64, size: u64) -> Result<i64> {
    let after = reader.seek(SeekFrom::Current(0))?;
    let remaining_bytes = (size - (after - current)) as i64;
    let size = remaining_bytes - HEADER_SIZE as i64;
    reader.seek(SeekFrom::Current(size))?;
    Ok(size)
}

pub fn skip_write<W: Write>(writer: &mut BufWriter<W>, size: u64) -> Result<u64> {
    for _ in 0..size {
        writer.write_u8(0)?;
    }
    Ok(size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fourcc() {
        let ftyp_fcc = 0x66747970;
        let ftyp_value = FourCC::from(ftyp_fcc);
        assert_eq!(ftyp_value.value, "ftyp");
        let ftyp_fcc2 = ftyp_value.into();
        assert_eq!(ftyp_fcc, ftyp_fcc2);
    }
}
