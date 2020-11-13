use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::convert::TryInto;
use std::io::{Read, Seek, SeekFrom, Write};

#[cfg(feature = "async")]
use {
    std::marker::Unpin,
    std::io::Cursor,
    tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncSeekExt}
};

use crate::*;

pub(crate) mod avc1;
pub(crate) mod co64;
pub(crate) mod ctts;
pub(crate) mod dinf;
pub(crate) mod edts;
pub(crate) mod elst;
pub(crate) mod ftyp;
pub(crate) mod hdlr;
pub(crate) mod mdhd;
pub(crate) mod mdia;
pub(crate) mod minf;
pub(crate) mod moov;
pub(crate) mod mp4a;
pub(crate) mod mvhd;
pub(crate) mod smhd;
pub(crate) mod stbl;
pub(crate) mod stco;
pub(crate) mod stsc;
pub(crate) mod stsd;
pub(crate) mod stss;
pub(crate) mod stsz;
pub(crate) mod stts;
pub(crate) mod tkhd;
pub(crate) mod trak;
pub(crate) mod vmhd;

pub use ftyp::FtypBox;
pub use moov::MoovBox;

pub const HEADER_SIZE: u64 = 8;
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

boxtype! {
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
    DrefBox => 0x64726566,
    UrlBox  => 0x75726C20,
    SmhdBox => 0x736d6864,
    Avc1Box => 0x61766331,
    AvcCBox => 0x61766343,
    Mp4aBox => 0x6d703461,
    EsdsBox => 0x65736473
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
    pub fn new(name: BoxType, size: u64) -> Self {
        Self { name, size }
    }

    // TODO: if size is 0, then this box is the last one in the file
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        // Create and read to buf.
        let mut buf = [0u8; 8]; // 8 bytes for box header.
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

    #[cfg(feature = "async")]
    pub async fn async_read<R>(reader: &mut R) -> Result<Self>
    where
        R: AsyncReadExt + Unpin
    {
        // Create and read to buf.
        let mut buf = [0u8; 8]; // 8 bytes for box header.
        reader.read(&mut buf).await?;

        // Get size.
        let s = buf[0..4].try_into().unwrap();
        let size = u32::from_be_bytes(s);

        // Get box type string.
        let t = buf[4..8].try_into().unwrap();
        let typ = u32::from_be_bytes(t);

        // Get largesize if size is 1
        if size == 1 {
            reader.read(&mut buf).await?;
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

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<u64> {
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

    #[cfg(feature = "async")]
    pub async fn async_write<W>(&self, writer: &mut W) -> Result<u64>
    where
        W: AsyncWriteExt + Unpin
    {
        let size = if self.size > u32::MAX as u64 {
            16
        } else {
            8
        };
        let mut buffer = vec![0u8; size];
        let hdr_size = self.write(&mut Cursor::new(&mut buffer))?;
        writer.write_all(&buffer).await?;
        Ok(hdr_size)
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

pub fn box_start<S: Seek>(seeker: &mut S) -> Result<u64> {
    Ok(seeker.seek(SeekFrom::Current(0))? - HEADER_SIZE)
}

#[cfg(feature = "async")]
pub async fn async_box_start<S: AsyncSeekExt + Unpin>(seeker: &mut S) -> Result<u64> {
    Ok(seeker.seek(SeekFrom::Current(0)).await? - HEADER_SIZE)
}

pub fn skip_bytes<S: Seek>(seeker: &mut S, size: u64) -> Result<()> {
    seeker.seek(SeekFrom::Current(size as i64))?;
    Ok(())
}

pub fn skip_bytes_to<S: Seek>(seeker: &mut S, pos: u64) -> Result<()> {
    seeker.seek(SeekFrom::Start(pos))?;
    Ok(())
}

#[cfg(feature = "async")]
pub async fn async_skip_bytes_to<S>(seeker: &mut S, pos: u64) -> Result<()>
where
    S: AsyncSeekExt + Unpin
{
    seeker.seek(SeekFrom::Start(pos)).await?;
    Ok(())
}

pub fn skip_box<S: Seek>(seeker: &mut S, size: u64) -> Result<()> {
    let start = box_start(seeker)?;
    skip_bytes_to(seeker, start + size)?;
    Ok(())
}

#[cfg(feature = "async")]
pub async fn async_skip_box<S>(seeker: &mut S, size: u64) -> Result<()>
where
    S: AsyncSeekExt + Unpin
{
    let start = async_box_start(seeker).await?;
    async_skip_bytes_to(seeker, start + size).await?;
    Ok(())
}

pub fn write_zeros<W: Write>(writer: &mut W, size: u64) -> Result<()> {
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
