use std::fmt;
use std::io::{BufReader, SeekFrom, Seek, Read, BufWriter, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::char::{decode_utf16, REPLACEMENT_CHARACTER};
use crate::{Error, read_box_header, BoxHeader, HEADER_SIZE};

const HEADER_EXT_SIZE: u64 = 4;

pub type Result<T> = std::result::Result<T, Error>;

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

#[derive(Debug, Default, PartialEq)]
pub struct FtypBox {
    pub major_brand: FourCC,
    pub minor_version: u32,
    pub compatible_brands: Vec<FourCC>,
}

#[derive(Debug, Default)]
pub struct MoovBox {
    pub mvhd: MvhdBox,
    pub traks: Vec<TrakBox>,
}

impl MoovBox {
    pub(crate) fn new() -> MoovBox {
        Default::default()
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct MvhdBox {
    pub version: u8,
    pub flags: u32,
    pub creation_time: u64,
    pub modification_time: u64,
    pub timescale: u32,
    pub duration: u64,
    pub rate: u32,
}

#[derive(Debug, Default)]
pub struct TrakBox {
    pub tkhd: Option<TkhdBox>,
    pub edts: Option<EdtsBox>,
    pub mdia: Option<MdiaBox>,
}

impl TrakBox {
    pub(crate) fn new() -> TrakBox {
        Default::default()
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct TkhdBox {
    pub version: u8,
    pub flags: u32,
    pub creation_time: u64,
    pub modification_time: u64,
    pub track_id: u32,
    pub duration: u64,
    pub layer:  u16,
    pub alternate_group: u16,
    pub volume: u16,
    pub matrix: Matrix,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Default, PartialEq)]
pub struct Matrix {
    pub a: i32,
    pub b: i32,
    pub u: i32,
    pub c: i32,
    pub d: i32,
    pub v: i32,
    pub x: i32,
    pub y: i32,
    pub w: i32,
}

#[derive(Debug, Default)]
pub struct EdtsBox {
    pub elst: Option<ElstBox>,
}

impl EdtsBox {
    pub(crate) fn new() -> EdtsBox {
        Default::default()
    }
}

#[derive(Debug, Default)]
pub struct ElstBox {
    pub version: u8,
    pub flags: u32,
    pub entry_count: u32,
    pub entries: Vec<ElstEntry>,
}

#[derive(Debug, Default)]
pub struct ElstEntry {
    pub segment_duration: u64,
    pub media_time: u64,
    pub media_rate: u16,
    pub media_rate_fraction: u16,
}

#[derive(Debug, Default)]
pub struct MdiaBox {
    pub mdhd: Option<MdhdBox>,
    pub hdlr: Option<HdlrBox>,
    pub minf: Option<MinfBox>,
}

impl MdiaBox {
    pub(crate) fn new() -> MdiaBox {
        Default::default()
    }
}

#[derive(Debug, Default)]
pub struct MdhdBox {
    pub version: u8,
    pub flags: u32,
    pub creation_time: u64,
    pub modification_time: u64,
    pub timescale: u32,
    pub duration: u64,
    pub language: u16,
    pub language_string: String,
}

#[derive(Debug, Default)]
pub struct HdlrBox {
    pub version: u8,
    pub flags: u32,
    pub handler_type: FourCC,
    pub name: String,
}

#[derive(Debug, Default)]
pub struct MinfBox {
    pub vmhd: Option<VmhdBox>,
    pub stbl: Option<StblBox>,
}

impl MinfBox {
    pub(crate) fn new() -> MinfBox {
        Default::default()
    }
}

#[derive(Debug, Default)]
pub struct VmhdBox {
    pub version: u8,
    pub flags: u32,
    pub graphics_mode: u16,
    pub op_color: RgbColor,
}

#[derive(Debug, Default)]
pub struct RgbColor {
    pub red: u16,
    pub green: u16,
    pub blue: u16,
}

#[derive(Debug, Default)]
pub struct StblBox {
    pub stts: Option<SttsBox>,
    pub stsd: Option<StsdBox>,
}

impl StblBox {
    pub(crate) fn new() -> StblBox {
        Default::default()
    }
}

#[derive(Debug, Default)]
pub struct SttsBox {
    pub version: u8,
    pub flags: u32,
    pub entry_count: u32,
    pub entries: Vec<SttsEntry>,
}

#[derive(Debug, Default)]
pub struct SttsEntry {
    pub sample_count: u32,
    pub sample_delta: u32,
}

#[derive(Debug)]
pub struct StsdBox {
    pub version: u8,
    pub flags: u32,
    pub entry_count: u32,
    pub entries: Vec<DumpBox>,
}

#[derive(Debug, PartialEq)]
pub struct DumpBox {
    pub name: BoxType,
    pub size: u64,
}

fn read_box_header_ext<R: Read>(reader: &mut BufReader<R>) -> Result<(u8, u32)> {
    let version = reader.read_u8().unwrap();
    let flags_a = reader.read_u8().unwrap();
    let flags_b = reader.read_u8().unwrap();
    let flags_c = reader.read_u8().unwrap();
    let flags = u32::from(flags_a) << 16 | u32::from(flags_b) << 8 | u32::from(flags_c);
    Ok((version, flags))
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for BoxHeader {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        if self.size > u32::MAX as u64 {
            writer.write_u32::<BigEndian>(1).unwrap();
            writer.write_u32::<BigEndian>(self.name.into()).unwrap();
            writer.write_u64::<BigEndian>(self.size).unwrap();
            Ok(16)
        } else {
            writer.write_u32::<BigEndian>(self.size as u32).unwrap();
            writer.write_u32::<BigEndian>(self.name.into()).unwrap();
            Ok(8)
        }
    }
}

fn write_box_header_ext<W: Write>(w: &mut BufWriter<W>, v: u8, f: u32) -> Result<u64> {
    let d = u32::from(v) << 24 | f;
    w.write_u32::<BigEndian>(d).unwrap();
    Ok(4)
}

impl Mp4Box for FtypBox {
    fn box_type(&self) -> BoxType {
        BoxType::FtypBox
    }

    fn box_size(&self) -> u64 {
        HEADER_SIZE + 8 + (4 * self.compatible_brands.len() as u64)
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for FtypBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let major = reader.read_u32::<BigEndian>().unwrap();
        let minor = reader.read_u32::<BigEndian>().unwrap();
        if size % 4 != 0 {
            return Err(Error::InvalidData("invalid ftyp size"));
        }
        let brand_count = (size - 16) / 4; // header + major + minor

        let mut brands = Vec::new();
        for _ in 0..brand_count {
            let b = reader.read_u32::<BigEndian>().unwrap();
            brands.push(From::from(b));
        }

        Ok(FtypBox {
            major_brand: From::from(major),
            minor_version: minor,
            compatible_brands: brands,
        })
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for FtypBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

        writer.write_u32::<BigEndian>((&self.major_brand).into()).unwrap();
        writer.write_u32::<BigEndian>(self.minor_version).unwrap();
        for b in self.compatible_brands.iter() {
            writer.write_u32::<BigEndian>(b.into()).unwrap();
        }
        Ok(size)
    }
}

impl Mp4Box for MoovBox {
    fn box_type(&self) -> BoxType {
        BoxType::MoovBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE + self.mvhd.box_size();
        for trak in self.traks.iter() {
            size += trak.box_size();
        }
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for MoovBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let mut moov = MoovBox::new();

        let mut start = 0u64;
        while start < size {

            // Get box header.
            let header = read_box_header(reader, start).unwrap();
            let BoxHeader{ name, size: s } = header;

            match name {
                BoxType::MvhdBox => {
                    moov.mvhd = MvhdBox::read_box(reader, s).unwrap();
                }
                BoxType::TrakBox => {
                    let trak = TrakBox::read_box(reader, s).unwrap();
                    moov.traks.push(trak);
                }
                BoxType::UdtaBox => {
                    start = s - HEADER_SIZE;
                }
                _ => break
            }
        }
        Ok(moov)
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for MoovBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

        self.mvhd.write_box(writer)?;
        for trak in self.traks.iter() {
            trak.write_box(writer)?;
        }
        Ok(0)
    }
}

impl Mp4Box for MvhdBox {
    fn box_type(&self) -> BoxType {
        BoxType::MvhdBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE;
        if self.version == 1 {
            size += 28;
        } else {
            assert_eq!(self.version, 0);
            size += 16;
        }
        size += 80;
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for MvhdBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

        let (version, flags) = read_box_header_ext(reader).unwrap();

        let (creation_time, modification_time, timescale, duration)
            = if version  == 1 {
                (
                    reader.read_u64::<BigEndian>().unwrap(),
                    reader.read_u64::<BigEndian>().unwrap(),
                    reader.read_u32::<BigEndian>().unwrap(),
                    reader.read_u64::<BigEndian>().unwrap(),
                )
            } else {
                assert_eq!(version, 0);
                (
                    reader.read_u32::<BigEndian>().unwrap() as u64,
                    reader.read_u32::<BigEndian>().unwrap() as u64,
                    reader.read_u32::<BigEndian>().unwrap(),
                    reader.read_u32::<BigEndian>().unwrap() as u64,
                )
            };
        let rate = reader.read_u32::<BigEndian>().unwrap();
        skip_read(reader, current, size);

        Ok(MvhdBox{
            version,
            flags,
            creation_time,
            modification_time,
            timescale,
            duration,
            rate,
        })
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for MvhdBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        if self.version == 1 {
            writer.write_u64::<BigEndian>(self.creation_time).unwrap();
            writer.write_u64::<BigEndian>(self.modification_time).unwrap();
            writer.write_u32::<BigEndian>(self.timescale).unwrap();
            writer.write_u64::<BigEndian>(self.duration).unwrap();
        } else {
            assert_eq!(self.version, 0);
            writer.write_u32::<BigEndian>(self.creation_time as u32).unwrap();
            writer.write_u32::<BigEndian>(self.modification_time as u32).unwrap();
            writer.write_u32::<BigEndian>(self.timescale).unwrap();
            writer.write_u32::<BigEndian>(self.duration as u32).unwrap();
        }
        writer.write_u32::<BigEndian>(self.rate).unwrap();

        // XXX volume, ...
        skip_write(writer, 76);

        Ok(size)
    }
}

impl Mp4Box for TrakBox {
    fn box_type(&self) -> BoxType {
        BoxType::TrakBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        if let Some(tkhd) = &self.tkhd {
            size += tkhd.box_size();
        }
        if let Some(edts) = &self.edts {
            size += edts.box_size();
        }
        if let Some(mdia) = &self.mdia {
            size += mdia.box_size();
        }
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for TrakBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.
        let mut trak = TrakBox::new();

        let start = 0u64;
        while start < size {
            // Get box header.
            let header = read_box_header(reader, start).unwrap();
            let BoxHeader{ name, size: s } = header;

            match name {
                BoxType::TkhdBox => {
                    let tkhd = TkhdBox::read_box(reader, s).unwrap();
                    trak.tkhd = Some(tkhd);
                }
                BoxType::EdtsBox => {
                    let edts = EdtsBox::read_box(reader, s).unwrap();
                    trak.edts = Some(edts);
                }
                BoxType::MdiaBox => {
                    let mdia = MdiaBox::read_box(reader, s).unwrap();
                    trak.mdia = Some(mdia);
                }
                _ => break
            }
        }
        skip_read(reader, current, size);

        Ok(trak)
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for TrakBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

        if let Some(tkhd) = &self.tkhd {
            tkhd.write_box(writer)?;
        }
        if let Some(edts) = &self.edts {
            edts.write_box(writer)?;
        }
        if let Some(mdia) = &self.mdia {
            mdia.write_box(writer)?;
        }

        Ok(size)
    }
}

impl Mp4Box for TkhdBox {
    fn box_type(&self) -> BoxType {
        BoxType::TkhdBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE;
        if self.version == 1 {
            size += 32;
        } else {
            assert_eq!(self.version, 0);
            size += 20;
        }
        size += 60;
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for TkhdBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

        let (version, flags) = read_box_header_ext(reader).unwrap();

        let (creation_time, modification_time, track_id, _, duration)
            = if version == 1 {
                (
                    reader.read_u64::<BigEndian>().unwrap(),
                    reader.read_u64::<BigEndian>().unwrap(),
                    reader.read_u32::<BigEndian>().unwrap(),
                    reader.read_u32::<BigEndian>().unwrap(),
                    reader.read_u64::<BigEndian>().unwrap(),
                )
        } else {
                assert_eq!(version, 0);
                (
                    reader.read_u32::<BigEndian>().unwrap() as u64,
                    reader.read_u32::<BigEndian>().unwrap() as u64,
                    reader.read_u32::<BigEndian>().unwrap(),
                    reader.read_u32::<BigEndian>().unwrap(),
                    reader.read_u32::<BigEndian>().unwrap() as u64,
                )
        };
        reader.read_u64::<BigEndian>().unwrap(); // reserved
        let layer = reader.read_u16::<BigEndian>().unwrap();
        let alternate_group = reader.read_u16::<BigEndian>().unwrap();
        let volume = reader.read_u16::<BigEndian>().unwrap();

        reader.read_u16::<BigEndian>().unwrap(); // reserved
        let matrix = Matrix{
            a: reader.read_i32::<byteorder::LittleEndian>().unwrap(),
            b: reader.read_i32::<BigEndian>().unwrap(),
            u: reader.read_i32::<BigEndian>().unwrap(),
            c: reader.read_i32::<BigEndian>().unwrap(),
            d: reader.read_i32::<BigEndian>().unwrap(),
            v: reader.read_i32::<BigEndian>().unwrap(),
            x: reader.read_i32::<BigEndian>().unwrap(),
            y: reader.read_i32::<BigEndian>().unwrap(),
            w: reader.read_i32::<BigEndian>().unwrap(),
        };

        let width = reader.read_u32::<BigEndian>().unwrap() >> 16;
        let height = reader.read_u32::<BigEndian>().unwrap() >> 16;

        skip_read(reader, current, size);

        Ok(TkhdBox {
            version,
            flags,
            creation_time,
            modification_time,
            track_id,
            duration,
            layer,
            alternate_group,
            volume,
            matrix,
            width,
            height,
        })
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for TkhdBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        if self.version == 1 {
            writer.write_u64::<BigEndian>(self.creation_time).unwrap();
            writer.write_u64::<BigEndian>(self.modification_time).unwrap();
            writer.write_u32::<BigEndian>(self.track_id).unwrap();
            writer.write_u32::<BigEndian>(0).unwrap(); // reserved
            writer.write_u64::<BigEndian>(self.duration).unwrap();
        } else {
            assert_eq!(self.version, 0);
            writer.write_u32::<BigEndian>(self.creation_time as u32).unwrap();
            writer.write_u32::<BigEndian>(self.modification_time as u32).unwrap();
            writer.write_u32::<BigEndian>(self.track_id).unwrap();
            writer.write_u32::<BigEndian>(0).unwrap(); // reserved
            writer.write_u32::<BigEndian>(self.duration as u32).unwrap();
        }

        writer.write_u64::<BigEndian>(0).unwrap(); // reserved
        writer.write_u16::<BigEndian>(self.layer).unwrap();
        writer.write_u16::<BigEndian>(self.alternate_group).unwrap();
        writer.write_u16::<BigEndian>(self.volume).unwrap();

        writer.write_u16::<BigEndian>(0).unwrap(); // reserved

        writer.write_i32::<byteorder::LittleEndian>(self.matrix.a).unwrap();
        writer.write_i32::<BigEndian>(self.matrix.b).unwrap();
        writer.write_i32::<BigEndian>(self.matrix.u).unwrap();
        writer.write_i32::<BigEndian>(self.matrix.c).unwrap();
        writer.write_i32::<BigEndian>(self.matrix.d).unwrap();
        writer.write_i32::<BigEndian>(self.matrix.v).unwrap();
        writer.write_i32::<BigEndian>(self.matrix.x).unwrap();
        writer.write_i32::<BigEndian>(self.matrix.y).unwrap();
        writer.write_i32::<BigEndian>(self.matrix.w).unwrap();

        writer.write_u32::<BigEndian>(self.width << 16).unwrap();
        writer.write_u32::<BigEndian>(self.height << 16).unwrap();

        Ok(size)
    }
}

impl Mp4Box for EdtsBox {
    fn box_type(&self) -> BoxType {
        BoxType::EdtsBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        if let Some(elst) = &self.elst {
            size += elst.box_size();
        }
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for EdtsBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.
        let mut edts = EdtsBox::new();

        let start = 0u64;
        while start < size {
            // Get box header.
            let header = read_box_header(reader, start).unwrap();
            let BoxHeader{ name, size: s } = header;

            match name {
                BoxType::ElstBox => {
                    let elst = ElstBox::read_box(reader, s).unwrap();
                    edts.elst = Some(elst);
                }
                _ => break
            }
        }
        skip_read(reader, current, size);

        Ok(edts)
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for EdtsBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

        if let Some(elst) = &self.elst {
            elst.write_box(writer)?;
        }

        Ok(size)
    }
}

impl Mp4Box for ElstBox {
    fn box_type(&self) -> BoxType {
        BoxType::ElstBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE;
        if self.version == 1 {
            size += self.entry_count as u64 * 20;
        } else {
            assert_eq!(self.version, 0);
            size += self.entry_count as u64 * 12;
        }
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for ElstBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

        let (version, flags) = read_box_header_ext(reader).unwrap();

        let entry_count = reader.read_u32::<BigEndian>().unwrap();
        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            let (segment_duration, media_time)
                = if version == 1 {
                    (
                        reader.read_u64::<BigEndian>().unwrap(),
                        reader.read_u64::<BigEndian>().unwrap(),
                    )
                } else {
                    (
                        reader.read_u32::<BigEndian>().unwrap() as u64,
                        reader.read_u32::<BigEndian>().unwrap() as u64,
                    )
                };

            let entry = ElstEntry{
                segment_duration,
                media_time,
                media_rate: reader.read_u16::<BigEndian>().unwrap(),
                media_rate_fraction: reader.read_u16::<BigEndian>().unwrap(),
            };
            entries.push(entry);
        }
        skip_read(reader, current, size);

        Ok(ElstBox {
            version,
            flags,
            entry_count,
            entries,
        })
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for ElstBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        assert_eq!(self.entry_count as usize, self.entries.len());
        writer.write_u32::<BigEndian>(self.entry_count).unwrap();
        for entry in self.entries.iter() {
            if self.version == 1 {
                writer.write_u64::<BigEndian>(entry.segment_duration).unwrap();
                writer.write_u64::<BigEndian>(entry.media_time).unwrap();
            } else {
                writer.write_u32::<BigEndian>(entry.segment_duration as u32).unwrap();
                writer.write_u32::<BigEndian>(entry.media_time as u32).unwrap();
            }
            writer.write_u16::<BigEndian>(entry.media_rate).unwrap();
            writer.write_u16::<BigEndian>(entry.media_rate_fraction).unwrap();
        }

        Ok(size)
    }
}

impl Mp4Box for MdiaBox {
    fn box_type(&self) -> BoxType {
        BoxType::MdiaBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        if let Some(mdhd) = &self.mdhd {
            size += mdhd.box_size();
        }
        if let Some(hdlr) = &self.hdlr {
            size += hdlr.box_size();
        }
        if let Some(minf) = &self.minf {
            size += minf.box_size();
        }
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for MdiaBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.
        let mut mdia = MdiaBox::new();

        let start = 0u64;
        while start < size {
            // Get box header.
            let header = read_box_header(reader, start).unwrap();
            let BoxHeader{ name, size: s } = header;

            match name {
                BoxType::MdhdBox => {
                    let mdhd = MdhdBox::read_box(reader, s).unwrap();
                    mdia.mdhd = Some(mdhd);
                }
                BoxType::HdlrBox => {
                    let hdlr = HdlrBox::read_box(reader, s).unwrap();
                    mdia.hdlr = Some(hdlr);
                }
                BoxType::MinfBox => {
                    let minf = MinfBox::read_box(reader, s).unwrap();
                    mdia.minf = Some(minf);
                }
                _ => break
            }
        }
        skip_read(reader, current, size);

        Ok(mdia)
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for MdiaBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

        if let Some(mdhd) = &self.mdhd {
            mdhd.write_box(writer)?;
        }
        if let Some(hdlr) = &self.hdlr {
            hdlr.write_box(writer)?;
        }
        if let Some(minf) = &self.minf {
            minf.write_box(writer)?;
        }

        Ok(size)
    }
}

impl Mp4Box for MdhdBox {
    fn box_type(&self) -> BoxType {
        BoxType::MdhdBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE;

        if self.version == 1 {
            size += 28;
        } else {
            assert_eq!(self.version, 0);
            size += 16;
        }
        size += 4;
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for MdhdBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

        let (version, flags) = read_box_header_ext(reader).unwrap();

        let (creation_time, modification_time, timescale, duration)
            = if version  == 1 {
                (
                    reader.read_u64::<BigEndian>().unwrap(),
                    reader.read_u64::<BigEndian>().unwrap(),
                    reader.read_u32::<BigEndian>().unwrap(),
                    reader.read_u64::<BigEndian>().unwrap(),
                )
            } else {
                assert_eq!(version, 0);
                (
                    reader.read_u32::<BigEndian>().unwrap() as u64,
                    reader.read_u32::<BigEndian>().unwrap() as u64,
                    reader.read_u32::<BigEndian>().unwrap(),
                    reader.read_u32::<BigEndian>().unwrap() as u64,
                )
            };
        let language = reader.read_u16::<BigEndian>().unwrap();
        let language_string = get_language_string(language);
        skip_read(reader, current, size);

        Ok(MdhdBox {
            version,
            flags,
            creation_time,
            modification_time,
            timescale,
            duration,
            language,
            language_string,
        })
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for MdhdBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        if self.version == 1 {
            writer.write_u64::<BigEndian>(self.creation_time).unwrap();
            writer.write_u64::<BigEndian>(self.modification_time).unwrap();
            writer.write_u32::<BigEndian>(self.timescale).unwrap();
            writer.write_u64::<BigEndian>(self.duration).unwrap();
        } else {
            assert_eq!(self.version, 0);
            writer.write_u32::<BigEndian>(self.creation_time as u32).unwrap();
            writer.write_u32::<BigEndian>(self.modification_time as u32).unwrap();
            writer.write_u32::<BigEndian>(self.timescale).unwrap();
            writer.write_u32::<BigEndian>(self.duration as u32).unwrap();
        }

        writer.write_u16::<BigEndian>(self.language).unwrap();
        writer.write_u16::<BigEndian>(0).unwrap(); // pre-defined

        Ok(size)
    }
}

fn get_language_string(language: u16) -> String {
    let mut lang: [u16; 3] = [0; 3];

    lang[0] = ((language >> 10) & 0x1F) + 0x60;
    lang[1] = ((language >> 5) & 0x1F) + 0x60;
    lang[2] = ((language) & 0x1F) + 0x60;

    // Decode utf-16 encoded bytes into a string.
    let lang_str = decode_utf16(lang.iter().cloned())
        .map(|r| r.unwrap_or(REPLACEMENT_CHARACTER))
        .collect::<String>();

    return lang_str;
}

impl Mp4Box for HdlrBox {
    fn box_type(&self) -> BoxType {
        BoxType::HdlrBox
    }

    fn box_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 20 + self.name.len() as u64 + 1
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for HdlrBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

        let (version, flags) = read_box_header_ext(reader).unwrap();

        reader.read_u32::<BigEndian>().unwrap(); // pre-defined
        let handler = reader.read_u32::<BigEndian>().unwrap();

        let n = reader.seek(SeekFrom::Current(12)).unwrap(); // 12 bytes reserved.

        let buf_size = (size - (n - current)) - HEADER_SIZE;
        let mut buf = vec![0u8; buf_size as usize];
        reader.read_exact(&mut buf).unwrap();

        let handler_string = match String::from_utf8(buf) {
            Ok(t) => {
                assert_eq!(t.len(), buf_size as usize);
                t
            },
            _ => String::from("null"),
        };

        skip_read(reader, current, size);

        Ok(HdlrBox {
            version,
            flags,
            handler_type: From::from(handler),
            name: handler_string,
        })
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for HdlrBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        writer.write_u32::<BigEndian>(0).unwrap(); // pre-defined
        writer.write_u32::<BigEndian>((&self.handler_type).into()).unwrap();

        // 12 bytes reserved
        for _ in 0..3 {
            writer.write_u32::<BigEndian>(0).unwrap();
        }

        writer.write(self.name.as_bytes()).unwrap();
        writer.write_u8(0).unwrap();

        Ok(size)
    }
}

impl Mp4Box for MinfBox {
    fn box_type(&self) -> BoxType {
        BoxType::MinfBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        if let Some(vmhd) = &self.vmhd {
            size += vmhd.box_size();
        }
        if let Some(stbl) = &self.stbl {
            size += stbl.box_size();
        }
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for MinfBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.
        let mut minf = MinfBox::new();

        let mut start = 0u64;
        while start < size {
            // Get box header.
            let header = read_box_header(reader, start).unwrap();
            let BoxHeader{ name, size: s } = header;

            match name {
                BoxType::VmhdBox => {
                    let vmhd = VmhdBox::read_box(reader, s).unwrap();
                    minf.vmhd = Some(vmhd);
                }
                BoxType::SmhdBox => {
                    start = s - HEADER_SIZE;
                }
                BoxType::DinfBox => {
                    start = s - HEADER_SIZE;
                }
                BoxType::StblBox => {
                    let stbl = StblBox::read_box(reader, s).unwrap();
                    minf.stbl = Some(stbl);
                }
                _ => break
            }
        }
        skip_read(reader, current, size);

        Ok(minf)
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for MinfBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

        if let Some(vmhd) = &self.vmhd {
            vmhd.write_box(writer)?;
        }
        if let Some(stbl) = &self.stbl {
            stbl.write_box(writer)?;
        }

        Ok(size)
    }
}

impl Mp4Box for VmhdBox {
    fn box_type(&self) -> BoxType {
        BoxType::VmhdBox
    }

    fn box_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 8
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for VmhdBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

        let (version, flags) = read_box_header_ext(reader).unwrap();

        let graphics_mode = reader.read_u16::<BigEndian>().unwrap();
        let op_color = RgbColor {
            red: reader.read_u16::<BigEndian>().unwrap(),
            green: reader.read_u16::<BigEndian>().unwrap(),
            blue: reader.read_u16::<BigEndian>().unwrap(),
        };
        skip_read(reader, current, size);

        Ok(VmhdBox {
            version,
            flags,
            graphics_mode,
            op_color,
        })
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for VmhdBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        writer.write_u16::<BigEndian>(self.graphics_mode).unwrap();
        writer.write_u16::<BigEndian>(self.op_color.red).unwrap();
        writer.write_u16::<BigEndian>(self.op_color.green).unwrap();
        writer.write_u16::<BigEndian>(self.op_color.blue).unwrap();

        Ok(size)
    }
}

impl Mp4Box for StblBox {
    fn box_type(&self) -> BoxType {
        BoxType::StblBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        if let Some(stts) = &self.stts {
            size += stts.box_size();
        }
        if let Some(stsd) = &self.stsd {
            size += stsd.box_size();
        }
        size
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for StblBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let mut stbl = StblBox::new();

        let start = 0u64;
        while start < size {
            // Get box header.
            let header = read_box_header(reader, start).unwrap();
            let BoxHeader{ name, size: s } = header;

            match name {
                BoxType::SttsBox => {
                    let stts = SttsBox::read_box(reader, s).unwrap();
                    stbl.stts = Some(stts);
                }
                BoxType::StsdBox => {
                    let stsd = StsdBox::read_box(reader, s).unwrap();
                    stbl.stsd = Some(stsd);
                }
                _ => break
            }
        }
        Ok(stbl)
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for StblBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

        if let Some(stts) = &self.stts {
            stts.write_box(writer)?;
        }
        if let Some(stsd) = &self.stsd {
            stsd.write_box(writer)?;
        }

        Ok(size)
    }
}

impl Mp4Box for SttsBox {
    fn box_type(&self) -> BoxType {
        BoxType::SttsBox
    }

    fn box_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 4 + (8 * self.entry_count as u64)
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for SttsBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> { 
        let current = reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

        let (version, flags) = read_box_header_ext(reader).unwrap();

        let entry_count = reader.read_u32::<BigEndian>().unwrap();
        let mut entries = Vec::with_capacity(entry_count as usize);
        for _i in 0..entry_count {
            let entry = SttsEntry {
                sample_count: reader.read_u32::<BigEndian>().unwrap(),
                sample_delta: reader.read_u32::<BigEndian>().unwrap(),
            };
            entries.push(entry);
        }
        skip_read(reader, current, size);

        Ok(SttsBox {
            version,
            flags,
            entry_count,
            entries,
        })
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for SttsBox {
    fn write_box(&self, writer: &mut BufWriter<W>) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write_box(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        writer.write_u32::<BigEndian>(self.entry_count).unwrap();
        for entry in self.entries.iter() {
            writer.write_u32::<BigEndian>(entry.sample_count).unwrap();
            writer.write_u32::<BigEndian>(entry.sample_delta).unwrap();
        }

        Ok(size)
    }
}

impl Mp4Box for StsdBox {
    fn box_type(&self) -> BoxType {
        BoxType::StsdBox
    }

    fn box_size(&self) -> u64 {
        // TODO
        0
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for StsdBox {
    fn read_box(reader: &mut BufReader<R>, size: u64) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

        let (version, flags) = read_box_header_ext(reader).unwrap();

        let entry_count = reader.read_u32::<BigEndian>().unwrap();
        let mut entries = Vec::with_capacity(entry_count as usize);

        let mut start = 0u64;
        while start < size {
            // Get box header.
            let header = read_box_header(reader, start).unwrap();
            let BoxHeader{ name, size: s } = header;

            match name {
                BoxType::Avc1Box => {}
                BoxType::Mp4aBox => {}
                _ => break
            }
            start += s - HEADER_SIZE;
            entries.push(DumpBox {name, size: s})
        }
        skip_read(reader, current, size);

        Ok(StsdBox {
            version,
            flags,
            entry_count,
            entries,
        })
    }
}

impl<W: Write> WriteBox<&mut BufWriter<W>> for StsdBox {
    fn write_box(&self, _writer: &mut BufWriter<W>) -> Result<u64> {
        // TODO
        Ok(0)
    }
}

fn skip_read<R: Read + Seek>(reader: &mut BufReader<R>, current: u64, size: u64) {
    let after = reader.seek(SeekFrom::Current(0)).unwrap();
    let remaining_bytes = (size - (after - current)) as i64;
    reader.seek(SeekFrom::Current(remaining_bytes - HEADER_SIZE as i64)).unwrap();
}

fn skip_write<W: Write>(writer: &mut BufWriter<W>, size: u64) {
    for _ in 0..size {
        writer.write_u8(0).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_fourcc() {
        let ftyp_fcc = 0x66747970;
        let ftyp_value = FourCC::from(ftyp_fcc);
        assert_eq!(ftyp_value.value, "ftyp");
        let ftyp_fcc2 = ftyp_value.into();
        assert_eq!(ftyp_fcc, ftyp_fcc2);
    }

    #[test]
    fn test_ftyp() {
        let src_box = FtypBox {
            major_brand: FourCC { value: String::from("isom") },
            minor_version: 0,
            compatible_brands: vec![
                FourCC { value: String::from("isom") },
                FourCC { value: String::from("iso2") },
                FourCC { value: String::from("avc1") },
                FourCC { value: String::from("mp41") },
            ]
        };
        let mut buf = Vec::new();
        {
            let mut writer = BufWriter::new(&mut buf);
            src_box.write_box(&mut writer).unwrap();
        }
        assert_eq!(buf.len(), src_box.box_size() as usize);

        {
            let mut reader = BufReader::new(Cursor::new(&buf));
            let header = read_box_header(&mut reader, 0).unwrap();
            assert_eq!(header.name, BoxType::FtypBox);
            assert_eq!(src_box.box_size(), header.size);

            let dst_box = FtypBox::read_box(&mut reader, header.size).unwrap();

            assert_eq!(src_box, dst_box);
        }
    }

    #[test]
    fn test_mvhd() {
        let src_box = MvhdBox {
            version: 0,
            flags: 0,
            creation_time: 100,
            modification_time: 200,
            timescale: 1000,
            duration: 634634,
            rate: 0x00010000,
        };
        let mut buf = Vec::new();
        {
            let mut writer = BufWriter::new(&mut buf);
            src_box.write_box(&mut writer).unwrap();
        }
        assert_eq!(buf.len(), src_box.box_size() as usize);

        {
            let mut reader = BufReader::new(Cursor::new(&buf));
            let header = read_box_header(&mut reader, 0).unwrap();
            assert_eq!(header.name, BoxType::MvhdBox);
            assert_eq!(src_box.box_size(), header.size);

            let dst_box = MvhdBox::read_box(&mut reader, header.size).unwrap();

            assert_eq!(src_box, dst_box);
        }
    }

    #[test]
    fn test_tkhd() {
        let src_box = TkhdBox {
            version: 0,
            flags: 0,
            creation_time: 100,
            modification_time: 200,
            track_id: 1,
            duration: 634634,
            layer: 0,
            alternate_group: 0,
            volume: 0x0100,
            matrix: Matrix {
                a: 0x00010000,
                b: 0,
                u: 0,
                c: 0,
                d: 0x00010000,
                v: 0,
                x: 0,
                y: 0,
                w: 0x40000000,
            },
            width: 512,
            height: 288,
        };
        let mut buf = Vec::new();
        {
            let mut writer = BufWriter::new(&mut buf);
            src_box.write_box(&mut writer).unwrap();
        }
        assert_eq!(buf.len(), src_box.box_size() as usize);

        {
            let mut reader = BufReader::new(Cursor::new(&buf));
            let header = read_box_header(&mut reader, 0).unwrap();
            assert_eq!(header.name, BoxType::TkhdBox);
            assert_eq!(src_box.box_size(), header.size);

            let dst_box = TkhdBox::read_box(&mut reader, header.size).unwrap();

            assert_eq!(src_box, dst_box);
        }
    }
}
