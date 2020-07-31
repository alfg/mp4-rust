use std::fmt;
use std::io::{Seek, SeekFrom, Read, Write};
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
mod smhd;
mod stbl;
mod stsd;
mod stts;
mod ctts;
mod stss;
mod stsc;
mod stsz;
mod stco;
mod co64;
mod avc;
mod mp4a;

pub use ftyp::FtypBox;
pub use moov::MoovBox;

const HEADER_SIZE: u64 = 8;
// const HEADER_LARGE_SIZE: u64 = 16;
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
    CttsBox => 0x63747473,
    StssBox => 0x73747373,
    StscBox => 0x73747363,
    StszBox => 0x7374737A,
    StcoBox => 0x7374636F,
    Co64Box => 0x636F3634,
    TrakBox => 0x7472616b,
    UdtaBox => 0x75647461,
    DinfBox => 0x64696e66,
    SmhdBox => 0x736d6864,
    Avc1Box => 0x61766331,
    AvcCBox => 0x61766343,
    Mp4aBox => 0x6d703461,
    EsdsBox => 0x65736473
}

impl fmt::Debug for BoxType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fourcc: FourCC = From::from(self.clone());
        write!(f, "{}", fourcc)
    }
}

impl fmt::Display for BoxType {
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
    fn box_type() -> BoxType;
    fn box_size(&self) -> u64;
}

pub trait ReadBox<T>: Sized {
    fn read_box(_: T, size: u64) -> Result<Self>;
}

pub trait WriteBox<T>: Sized {
    fn write_box(&self, _: T) -> Result<u64>;
}

#[derive(Debug, Clone, Copy)]
pub struct BoxHeader {
    pub name: BoxType,
    pub size: u64,
}

impl BoxHeader {
    fn new(name: BoxType, size: u64) -> Self {
        Self { name, size }
    }

    // TODO: if size is 0, then this box is the last one in the file
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        // Create and read to buf.
        let mut buf = [0u8;8]; // 8 bytes for box header.
        reader.read(&mut buf)?;

        // Get size.
        let s = buf[0..4].try_into().unwrap();
        let size = u32::from_be_bytes(s);

        // Get box type string.
        let t = buf[4..8].try_into().unwrap();
        let typ = u32::from_be_bytes(t);

        // Get largesize if size is 1
        if size == 1 {
            reader.read(&mut buf)?;
            let s = buf.try_into().unwrap();
            let largesize = u64::from_be_bytes(s);

            Ok(BoxHeader {
                name: BoxType::from(typ),
                size: largesize,
            })
        } else {
            Ok(BoxHeader {
                name: BoxType::from(typ),
                size: size as u64,
            })
        }
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<u64> {
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

pub fn read_box_header_ext<R: Read>(reader: &mut R) -> Result<(u8, u32)> {
    let version = reader.read_u8()?;
    let flags = reader.read_u24::<BigEndian>()?;
    Ok((version, flags))
}

pub fn write_box_header_ext<W: Write>(w: &mut W, v: u8, f: u32) -> Result<u64> {
    w.write_u8(v)?;
    w.write_u24::<BigEndian>(f)?;
    Ok(4)
}

pub fn get_box_start<R: Seek>(reader: &mut R) -> Result<u64> {
    Ok(reader.seek(SeekFrom::Current(0))? - HEADER_SIZE)
}

pub fn skip_read<R: Read + Seek>(reader: &mut R, size: i64) -> Result<()> {
    assert!(size >= 0);
    reader.seek(SeekFrom::Current(size))?;
    Ok(())
}

pub fn skip_read_to<R: Read + Seek>(reader: &mut R, pos: u64) -> Result<()> {
    reader.seek(SeekFrom::Start(pos))?;
    Ok(())
}

pub fn skip_box<R: Read + Seek>(reader: &mut R, size: u64) -> Result<()> {
    let start = get_box_start(reader)?;
    skip_read_to(reader, start + size)?;
    Ok(())
}

pub fn skip_write<W: Write>(writer: &mut W, size: u64) -> Result<()> {
    for _ in 0..size {
        writer.write_u8(0)?;
    }
    Ok(())
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
