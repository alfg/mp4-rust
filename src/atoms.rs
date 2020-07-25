use std::fmt;
use std::io::{BufReader, SeekFrom, Seek, Read};
use byteorder::{BigEndian, ReadBytesExt};
use std::char::{decode_utf16, REPLACEMENT_CHARACTER};
use crate::{Error, read_box_header, BoxHeader, HEADER_SIZE};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, PartialEq)]
pub enum BoxType {
    FtypBox,
    MvhdBox,
    FreeBox,
    MdatBox,
    MoovBox,
    MoofBox,
    TkhdBox,
    EdtsBox,
    MdiaBox,
    ElstBox,
    MdhdBox,
    HdlrBox,
    MinfBox,
    VmhdBox,
    StblBox,
    StsdBox,
    SttsBox,
    TrakBox,
    UdtaBox,
    DinfBox,
    SmhdBox,
    Avc1Box,
    Mp4aBox,
    UnknownBox(u32),
}

impl From<u32> for BoxType {
    fn from(t: u32) -> BoxType {
        use self::BoxType::*;
        match t {
            0x66747970 => FtypBox,
            0x6d766864 => MvhdBox,
            0x66726565 => FreeBox,
            0x6d646174 => MdatBox,
            0x6d6f6f76 => MoovBox,
            0x6d6f6f66 => MoofBox ,
            0x746b6864 => TkhdBox,
            0x65647473 => EdtsBox,
            0x6d646961 => MdiaBox,
            0x656c7374 => ElstBox,
            0x6d646864 => MdhdBox,
            0x68646c72 => HdlrBox,
            0x6d696e66 => MinfBox,
            0x766d6864 => VmhdBox,
            0x7374626c => StblBox,
            0x73747364 => StsdBox,
            0x73747473 => SttsBox,
            0x7472616b => TrakBox,
            0x75647461 => UdtaBox,
            0x64696e66 => DinfBox,
            0x736d6864 => SmhdBox,
            0x61766331 => Avc1Box,
            0x6d703461 => Mp4aBox,
            _ => UnknownBox(t),
        }
    }
}

impl Into<u32> for BoxType {
    fn into(self) -> u32 {
        use self::BoxType::*;
        match self {
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
            Mp4aBox => 0x6d703461,

            UnknownBox(t) => t,
        }
    }
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
    fn from(number: u32) -> FourCC {
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

pub trait ReadBox<T>: Sized {
    fn read_box(_: T, offset: u64, size: u32) -> Result<Self>;
}

#[derive(Debug, Default)]
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

#[derive(Debug, Default)]
pub struct MvhdBox {
    pub version: u8,
    pub flags: u32,
    pub creation_time: u32,
    pub modification_time: u32,
    pub timescale: u32,
    pub duration: u32,
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

#[derive(Debug, Default)]
pub struct TkhdBox {
    pub version: u8,
    pub flags: u32,
    pub creation_time: u32,
    pub modification_time: u32,
    pub track_id: u32,
    pub duration: u64,
    pub layer:  u16,
    pub alternate_group: u16,
    pub volume: u16,
    pub matrix: Matrix,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Default)]
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
    pub version: u32,
    pub entry_count: u32,
    pub entries: Vec<ElstEntry>,
}

#[derive(Debug, Default)]
pub struct ElstEntry {
    pub segment_duration: u32,
    pub media_time: u32,
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
    pub creation_time: u32,
    pub modification_time: u32,
    pub timescale: u32,
    pub duration: u32,
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
    pub op_color: u16,
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
    pub sample_counts: Vec<u32>,
    pub sample_deltas: Vec<u32>,
}

#[derive(Debug, Default)]
pub struct StsdBox {
    pub version: u8,
    pub flags: u32,
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for FtypBox {
    fn read_box(reader: &mut BufReader<R>, _offset: u64, size: u32) -> Result<Self> {
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

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for MoovBox {
    fn read_box(reader: &mut BufReader<R>, _offset: u64, size: u32) -> Result<Self> {
        let mut moov = MoovBox::new();

        let mut start = 0u64;
        while start < size as u64 {

            // Get box header.
            let header = read_box_header(reader, start).unwrap();
            let BoxHeader{ name, size: s, offset: _ } = header;

            match name {
                BoxType::MvhdBox => {
                    moov.mvhd = MvhdBox::read_box(reader, 0, s as u32).unwrap();
                }
                BoxType::TrakBox => {
                    let trak = TrakBox::read_box(reader, 0, s as u32).unwrap();
                    moov.traks.push(trak);
                }
                BoxType::UdtaBox => {
                    start = (s as u32 - HEADER_SIZE) as u64;
                }
                _ => break
            }
        }
        Ok(moov)
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for MvhdBox {
    fn read_box(reader: &mut BufReader<R>, _offset: u64, size: u32) -> Result<Self> {
        let current =  reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

        let version = reader.read_u8().unwrap();
        let flags_a = reader.read_u8().unwrap();
        let flags_b = reader.read_u8().unwrap();
        let flags_c = reader.read_u8().unwrap();
        let flags = u32::from(flags_a) << 16 | u32::from(flags_b) << 8 | u32::from(flags_c);
        let creation_time = reader.read_u32::<BigEndian>().unwrap();
        let modification_time = reader.read_u32::<BigEndian>().unwrap();
        let timescale = reader.read_u32::<BigEndian>().unwrap();
        let duration = reader.read_u32::<BigEndian>().unwrap();
        let rate = reader.read_u32::<BigEndian>().unwrap();
        skip(reader, current, size);

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

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for TrakBox {
    fn read_box(reader: &mut BufReader<R>, _offset: u64, size: u32) -> Result<Self> {
        let current =  reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.
        let mut trak = TrakBox::new();

        let start = 0u64;
        while start < size as u64 {
            // Get box header.
            let header = read_box_header(reader, start).unwrap();
            let BoxHeader{ name, size: s, offset: _ } = header;

            match name {
                BoxType::TkhdBox => {
                    let tkhd = TkhdBox::read_box(reader, 0, s as u32).unwrap();
                    trak.tkhd = Some(tkhd);
                }
                BoxType::EdtsBox => {
                    let edts = EdtsBox::read_box(reader, 0, s as u32).unwrap();
                    trak.edts = Some(edts);
                }
                BoxType::MdiaBox => {
                    let mdia = MdiaBox::read_box(reader, 0, s as u32).unwrap();
                    trak.mdia = Some(mdia);
                }
                _ => break
            }
        }
        skip(reader, current, size);

        Ok(trak)
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for TkhdBox {
    fn read_box(reader: &mut BufReader<R>, _offset: u64, size: u32) -> Result<Self> {
        let current =  reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

        let version = reader.read_u8().unwrap();
        let flags_a = reader.read_u8().unwrap();
        let flags_b = reader.read_u8().unwrap();
        let flags_c = reader.read_u8().unwrap();
        let flags = u32::from(flags_a) << 16 | u32::from(flags_b) << 8 | u32::from(flags_c);
        let creation_time = reader.read_u32::<BigEndian>().unwrap();
        let modification_time = reader.read_u32::<BigEndian>().unwrap();
        let track_id = reader.read_u32::<BigEndian>().unwrap();
        let duration = reader.read_u64::<BigEndian>().unwrap();
        reader.read_u64::<BigEndian>().unwrap(); // skip.
        let layer = reader.read_u16::<BigEndian>().unwrap();
        let alternate_group = reader.read_u16::<BigEndian>().unwrap();
        let volume = reader.read_u16::<BigEndian>().unwrap() >> 8;

        reader.read_u8().unwrap(); // skip.
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

        let width = reader.read_u32::<BigEndian>().unwrap() >> 8;
        let height = reader.read_u32::<BigEndian>().unwrap() >> 8;
        skip(reader, current, size);

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

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for EdtsBox {
    fn read_box(reader: &mut BufReader<R>, _offset: u64, size: u32) -> Result<Self> {
        let current =  reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.
        let mut edts = EdtsBox::new();

        let start = 0u64;
        while start < size as u64 {
            // Get box header.
            let header = read_box_header(reader, start).unwrap();
            let BoxHeader{ name, size: s, offset: _ } = header;

            match name {
                BoxType::ElstBox => {
                    let elst = ElstBox::read_box(reader, 0, s as u32).unwrap();
                    edts.elst = Some(elst);
                }
                _ => break
            }
        }
        skip(reader, current, size);

        Ok(edts)
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for ElstBox {
    fn read_box(reader: &mut BufReader<R>, _offset: u64, size: u32) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

        let version = reader.read_u32::<BigEndian>().unwrap();
        let entry_count = reader.read_u32::<BigEndian>().unwrap();

        let mut entries = Vec::new();

        for _i in 0..entry_count {
            let entry = ElstEntry{
                segment_duration: reader.read_u32::<BigEndian>().unwrap(),
                media_time: reader.read_u32::<BigEndian>().unwrap(),
                media_rate: reader.read_u16::<BigEndian>().unwrap(),
                media_rate_fraction: reader.read_u16::<BigEndian>().unwrap(),
            };
            entries.push(entry);
        }
        skip(reader, current, size);

        Ok(ElstBox {
            version,
            entry_count,
            entries,
        })
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for MdiaBox {
    fn read_box(reader: &mut BufReader<R>, _offset: u64, size: u32) -> Result<Self> {
        let current =  reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.
        let mut mdia = MdiaBox::new();

        let start = 0u64;
        while start < size as u64 {
            // Get box header.
            let header = read_box_header(reader, start).unwrap();
            let BoxHeader{ name, size: s, offset: _ } = header;

            match name {
                BoxType::MdhdBox => {
                    let mdhd = MdhdBox::read_box(reader, 0, s as u32).unwrap();
                    mdia.mdhd = Some(mdhd);
                }
                BoxType::HdlrBox => {
                    let hdlr = HdlrBox::read_box(reader, 0, s as u32).unwrap();
                    mdia.hdlr = Some(hdlr);
                }
                BoxType::MinfBox => {
                    let minf = MinfBox::read_box(reader, 0, s as u32).unwrap();
                    mdia.minf = Some(minf);
                }
                _ => break
            }
        }
        skip(reader, current, size);

        Ok(mdia)
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for MdhdBox {
    fn read_box(reader: &mut BufReader<R>, _offset: u64, size: u32) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

        let version = reader.read_u8().unwrap();
        let flags_a = reader.read_u8().unwrap();
        let flags_b = reader.read_u8().unwrap();
        let flags_c = reader.read_u8().unwrap();
        let flags = u32::from(flags_a) << 16 | u32::from(flags_b) << 8 | u32::from(flags_c);
        let creation_time = reader.read_u32::<BigEndian>().unwrap();
        let modification_time = reader.read_u32::<BigEndian>().unwrap();
        let timescale = reader.read_u32::<BigEndian>().unwrap();
        let duration = reader.read_u32::<BigEndian>().unwrap();
        let language = reader.read_u16::<BigEndian>().unwrap();
        let language_string = get_language_string(language);
        skip(reader, current, size);

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

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for HdlrBox {
    fn read_box(reader: &mut BufReader<R>, _offset: u64, size: u32) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

        let version = reader.read_u8().unwrap();
        let flags_a = reader.read_u8().unwrap();
        let flags_b = reader.read_u8().unwrap();
        let flags_c = reader.read_u8().unwrap();
        let flags = u32::from(flags_a) << 16 | u32::from(flags_b) << 8 | u32::from(flags_c);
        reader.read_u32::<BigEndian>().unwrap(); // skip.
        let handler = reader.read_u32::<BigEndian>().unwrap();

        let n = reader.seek(SeekFrom::Current(12)).unwrap(); // 12 bytes reserved.
        let buf_size = (size as u64 - (n - current)) - HEADER_SIZE as u64;
        let mut buf = vec![0u8; buf_size as usize];
        reader.read_exact(&mut buf).unwrap();

        let handler_string = match String::from_utf8(buf) {
            Ok(t) => t,
            _ => String::from("null"),
        };
        skip(reader, current, size);

        Ok(HdlrBox {
            version,
            flags,
            handler_type: From::from(handler),
            name: handler_string,
        })
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for MinfBox {
    fn read_box(reader: &mut BufReader<R>, _offset: u64, size: u32) -> Result<Self> {
        let current =  reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.
        let mut minf = MinfBox::new();

        let mut start = 0u64;
        while start < size as u64 {
            // Get box header.
            let header = read_box_header(reader, start).unwrap();
            let BoxHeader{ name, size: s, offset: _ } = header;

            match name {
                BoxType::VmhdBox => {
                    let vmhd = VmhdBox::read_box(reader, 0, s as u32).unwrap();
                    minf.vmhd = Some(vmhd);
                }
                BoxType::SmhdBox => {
                    start = (s as u32 - HEADER_SIZE) as u64;
                }
                BoxType::DinfBox => {
                    start = (s as u32 - HEADER_SIZE) as u64;
                }
                BoxType::StblBox => {
                    let stbl = StblBox::read_box(reader, 0, s as u32).unwrap();
                    minf.stbl = Some(stbl);
                }
                _ => break
            }
        }
        skip(reader, current, size);

        Ok(minf)
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for VmhdBox {
    fn read_box(reader: &mut BufReader<R>, _offset: u64, size: u32) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

        let version = reader.read_u8().unwrap();
        let flags_a = reader.read_u8().unwrap();
        let flags_b = reader.read_u8().unwrap();
        let flags_c = reader.read_u8().unwrap();
        let flags = u32::from(flags_a) << 16 | u32::from(flags_b) << 8 | u32::from(flags_c);
        let graphics_mode = reader.read_u16::<BigEndian>().unwrap();
        let op_color = reader.read_u16::<BigEndian>().unwrap();
        skip(reader, current, size);

        Ok(VmhdBox {
            version,
            flags,
            graphics_mode,
            op_color,
        })
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for StblBox {
    fn read_box(reader: &mut BufReader<R>, _offset: u64, size: u32) -> Result<Self> {
        let mut stbl = StblBox::new();

        let start = 0u64;
        while start < size as u64 {
            // Get box header.
            let header = read_box_header(reader, start).unwrap();
            let BoxHeader{ name, size: s, offset: _ } = header;

            match name {
                BoxType::StsdBox => {
                    let stsd = StsdBox::read_box(reader, 0, s as u32).unwrap();
                    stbl.stsd = Some(stsd);
                }
                BoxType::SttsBox => {
                    let stts = SttsBox::read_box(reader, 0, s as u32).unwrap();
                    stbl.stts = Some(stts);
                }
                _ => break
            }
        }
        Ok(stbl)
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for SttsBox {
    fn read_box(reader: &mut BufReader<R>, _offset: u64, size: u32) -> Result<Self> { 
        let current = reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

        let version = reader.read_u8().unwrap();
        let flags_a = reader.read_u8().unwrap();
        let flags_b = reader.read_u8().unwrap();
        let flags_c = reader.read_u8().unwrap();
        let flags = u32::from(flags_a) << 16 | u32::from(flags_b) << 8 | u32::from(flags_c);
        let entry_count = reader.read_u32::<BigEndian>().unwrap();
        let mut sample_counts = Vec::new();
        let mut sample_deltas = Vec::new();

        for _i in 0..entry_count {
            let sc = reader.read_u32::<BigEndian>().unwrap();
            let sd = reader.read_u32::<BigEndian>().unwrap();
            sample_counts.push(sc);
            sample_deltas.push(sd);
        }
        skip(reader, current, size);

        Ok(SttsBox {
            version,
            flags,
            entry_count,
            sample_counts,
            sample_deltas,
        })
    }
}

impl<R: Read + Seek> ReadBox<&mut BufReader<R>> for StsdBox {
    fn read_box(reader: &mut BufReader<R>, _offset: u64, size: u32) -> Result<Self> {
        let current = reader.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

        let version = reader.read_u8().unwrap();
        let flags_a = reader.read_u8().unwrap();
        let flags_b = reader.read_u8().unwrap();
        let flags_c = reader.read_u8().unwrap();
        let flags = u32::from(flags_a) << 16 | u32::from(flags_b) << 8 | u32::from(flags_c);
        reader.read_u32::<BigEndian>().unwrap(); // skip.

        let mut start = 0u64;
        while start < size as u64 {
            // Get box header.
            let header = read_box_header(reader, start).unwrap();
            let BoxHeader{ name, size: s, offset: _ } = header;

            match name {
                BoxType::Avc1Box => {
                    start = (s as u32 - HEADER_SIZE) as u64;
                }
                BoxType::Mp4aBox => {
                    start = (s as u32 - HEADER_SIZE) as u64;
                }
                _ => break
            }
        }
        skip(reader, current, size);

        Ok(StsdBox {
            version,
            flags,
        })
    }
}

fn skip<R: Read + Seek>(reader: &mut BufReader<R>, current: u64, size: u32) {
    let after = reader.seek(SeekFrom::Current(0)).unwrap();
    let remaining_bytes = (size as u64 - (after - current)) as i64;
    reader.seek(SeekFrom::Current(remaining_bytes - HEADER_SIZE as i64)).unwrap();
}
