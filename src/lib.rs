extern crate byteorder;

use std::io::prelude::*;
use std::io::{BufReader, Read, SeekFrom};
use std::fs::File;
use std::fmt;
use std::convert::TryInto;
use std::char::{decode_utf16, REPLACEMENT_CHARACTER};
use byteorder::{ReadBytesExt, BigEndian};

const HEADER_SIZE: u32 = 8;

#[derive(Debug)]
pub enum Error {
    InvalidData(&'static str),
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum TrackType {
    Audio,
    Video,
    Metadata,
    Unknown,
}

#[derive(Debug, Default)]
pub struct BMFF {
    pub ftyp: FtypBox,
    pub moov: Option<MoovBox>,
    pub size: u64,
}

impl BMFF {
    fn new() -> BMFF {
        Default::default()
    }
}

#[derive(Debug, Default)]
struct BMFFBox {
    head: BoxHeader,
}

impl BMFFBox {
    fn new() -> BMFFBox {
        Default::default()
    }
}

#[derive(Debug, Default)]
struct BoxHeader {
    name: String,
    size: u64,
    offset: u64,
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
    fn new() -> MoovBox {
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
    fn new() -> TrakBox {
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
    a: i32,
    b: i32,
    u: i32,
    c: i32,
    d: i32,
    v: i32,
    x: i32,
    y: i32,
    w: i32,
}

#[derive(Debug, Default)]
pub struct EdtsBox {
    pub elst: Option<ElstBox>,
}

impl EdtsBox {
    fn new() -> EdtsBox {
        Default::default()
    }
}

#[derive(Debug, Default)]
pub struct ElstBox {
    pub version: u32,
    pub entry_count: u32,
    pub entries: Vec<ElstEntry>,
}

impl ElstBox {
    fn new() -> ElstBox {
        Default::default()
    }
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
    fn new() -> MdiaBox {
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

impl MdhdBox {
    fn new() -> MdhdBox {
        Default::default()
    }
}

#[derive(Debug, Default)]
pub struct HdlrBox {
    pub version: u8,
    pub flags: u32,
    pub handler_type: FourCC,
    pub name: String,
}

impl HdlrBox {
    fn new() -> HdlrBox {
        Default::default()
    }
}

#[derive(Debug, Default)]
pub struct MinfBox {
    pub vmhd: Option<VmhdBox>,
    pub stbl: Option<StblBox>,
}

impl MinfBox {
    fn new() -> MinfBox {
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

impl VmhdBox {
    fn new() -> VmhdBox {
        Default::default()
    }
}

#[derive(Debug, Default)]
pub struct StblBox {
    pub stts: Option<SttsBox>,
    pub stsd: Option<StsdBox>,
}

impl StblBox {
    fn new() -> StblBox {
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

impl SttsBox {
    fn new() -> SttsBox {
        Default::default()
    }
}

#[derive(Debug, Default)]
pub struct StsdBox {
    pub version: u8,
    pub flags: u32,
}

impl StsdBox {
    fn new() -> StsdBox {
        Default::default()
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

impl fmt::Debug for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

pub fn read_mp4(f: File) -> Result<BMFF> {

    // Open file and read boxes.
    let bmff = read_boxes(f).unwrap();

    Ok(bmff)
}

fn read_boxes(f: File) -> Result<BMFF> {
    let filesize = f.metadata().unwrap().len();
    let mut reader = BufReader::new(f);
    let mut bmff = BMFF::new();
    bmff.size  =  filesize;

    let mut start = 0u64;
    while start < filesize {

        // Get box header.
        let header = read_box_header(&mut reader, start).unwrap();
        let BoxHeader{ name, size, offset } = header;

        let mut b = BMFFBox::new();
        b.head = BoxHeader{
            name: name.try_into().unwrap(),
            size: size as u64,
            offset: offset as u64,
        };

        // Match and parse the filetype.
        match b.head.name.as_ref() {
            "ftyp" => {
                let ftyp = parse_ftyp_box(&mut reader, 0, size as u32).unwrap();
                bmff.ftyp = ftyp;
            }
            "free" => {
                start = 0;
            }
            "mdat" => {
                start = (size as u32 - HEADER_SIZE) as u64;
            }
            "moov" => {
//                start = (size as u32 - HEADER_SIZE) as u64;
                let moov = parse_moov_box(&mut reader, 0, size as u32).unwrap();
                bmff.moov = Some(moov);
            }
            "moof" => {
                start = (size as u32 - HEADER_SIZE) as u64;
            }
            _ => break
        };
    }
    Ok(bmff)
}

fn read_box_header(reader: &mut BufReader<File>, start: u64) -> Result<BoxHeader> {
    // Seek to offset.
    let _r = reader.seek(SeekFrom::Current(start as i64));

    // Create and read to buf.
    let mut buf = [0u8;8]; // 8 bytes for box header.
    let _n = reader.read(&mut buf);

    // Get size.
    let s = buf[0..4].try_into().unwrap();
    let size = u32::from_be_bytes(s);

    // TODO: Err if size is 0.
    // if size == 0 { break; }
    
    // Get box type string.
    let t = buf[4..8].try_into().unwrap();
    let typ = match std::str::from_utf8(t) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    let offset = match size {
        1 => 4 + 4 + 8,
        _ => 4 + 4,
    };
    Ok(BoxHeader {
        name: typ.try_into().unwrap(),
        size: size as u64,
        offset: offset as u64,
    })
}

fn parse_ftyp_box(f: &mut BufReader<File>, _offset: u64, size: u32) -> Result<FtypBox> {
    let major = f.read_u32::<BigEndian>().unwrap();
    let minor = f.read_u32::<BigEndian>().unwrap();
    if size % 4 != 0 {
        return Err(Error::InvalidData("invalid ftyp size"));
    }
    let brand_count = (size - 16) / 4; // header + major + minor

    let mut brands = Vec::new();
    for _ in 0..brand_count {
        let b = f.read_u32::<BigEndian>().unwrap();
        brands.push(From::from(b));
    }

    Ok(FtypBox {
        major_brand: From::from(major),
        minor_version: minor,
        compatible_brands: brands,
    })
}

fn parse_moov_box(f: &mut BufReader<File>, _offset: u64, size: u32) -> Result<MoovBox> {
    let mut moov = MoovBox::new();

    let mut start = 0u64;
    while start < size as u64 {

        // Get box header.
        let header = read_box_header(f, start).unwrap();
        let BoxHeader{ name, size: s, offset } = header;

        let mut b = BMFFBox::new();
        b.head = BoxHeader{
            name: name.try_into().unwrap(),
            size: s as u64,
            offset: offset as u64,
        };


        match b.head.name.as_ref() {
            "mvhd" => {
                moov.mvhd = parse_mvhd_box(f, 0, s as u32).unwrap();
            }
            "trak" => {
                let trak = parse_trak_box(f, 0, s as u32).unwrap();
                moov.traks.push(trak);
                // start = (s as u32 - HEADER_SIZE) as u64;
            }
            "udta" => {
                println!("found udta");
                start = (s as u32 - HEADER_SIZE) as u64;
            }
            _ => break
        }
    }
    Ok(moov)
}

fn parse_mvhd_box(f: &mut BufReader<File>, _offset: u64, size: u32) -> Result<MvhdBox> {
    let current =  f.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

    let version = f.read_u8().unwrap();
    let flags_a = f.read_u8().unwrap();
    let flags_b = f.read_u8().unwrap();
    let flags_c = f.read_u8().unwrap();
    let flags = u32::from(flags_a) << 16 | u32::from(flags_b) << 8 | u32::from(flags_c);
    let creation_time = f.read_u32::<BigEndian>().unwrap();
    let modification_time = f.read_u32::<BigEndian>().unwrap();
    let timescale = f.read_u32::<BigEndian>().unwrap();
    let duration = f.read_u32::<BigEndian>().unwrap();
    let rate = f.read_u32::<BigEndian>().unwrap();

    // Skip remaining bytes.
    let after =  f.seek(SeekFrom::Current(0)).unwrap();
    let remaining_bytes = (size as u64 - (after - current)) as i64;
    f.seek(SeekFrom::Current(remaining_bytes - HEADER_SIZE as i64)).unwrap();

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

fn parse_trak_box(f: &mut BufReader<File>, _offset: u64, size: u32) -> Result<TrakBox> {
    let current =  f.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.
    let mut trak = TrakBox::new();

    let start = 0u64;
    while start < size as u64 {
        // Get box header.
         let header = read_box_header(f, start).unwrap();
         let BoxHeader{ name, size: s, offset } = header;

         let mut b = BMFFBox::new();
         b.head = BoxHeader{
             name: name.try_into().unwrap(),
             size: s as u64,
             offset: offset as u64,
         };

         match b.head.name.as_ref() {
             "tkhd" => {
                 let tkhd = parse_tkhd_box(f, 0, s as u32).unwrap();
                 trak.tkhd = Some(tkhd);
             }
             "edts" => {
                 let edts = parse_edts_box(f, 0, s as u32).unwrap();
                 trak.edts = Some(edts);
             }
             "mdia" => {
                 println!("found mdia");
                 let mdia = parse_mdia_box(f, 0, s as u32).unwrap();
                 trak.mdia = Some(mdia);
//                 start = (s as u32 - HEADER_SIZE) as u64;
             }
             _ => break
         }
    }

    // Skip remaining bytes.
    let after =  f.seek(SeekFrom::Current(0)).unwrap();
    let remaining_bytes = (size as u64 - (after - current)) as i64;
    f.seek(SeekFrom::Current(remaining_bytes - HEADER_SIZE as i64)).unwrap();
    Ok(trak)
}


fn parse_tkhd_box(f: &mut BufReader<File>, _offset: u64, size: u32) -> Result<TkhdBox> {
    let current =  f.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

    let version = f.read_u8().unwrap();
    let flags_a = f.read_u8().unwrap();
    let flags_b = f.read_u8().unwrap();
    let flags_c = f.read_u8().unwrap();
    let flags = u32::from(flags_a) << 16 | u32::from(flags_b) << 8 | u32::from(flags_c);
    let creation_time = f.read_u32::<BigEndian>().unwrap();
    let modification_time = f.read_u32::<BigEndian>().unwrap();
    let track_id = f.read_u32::<BigEndian>().unwrap();
    let duration = f.read_u64::<BigEndian>().unwrap();
    f.read_u64::<BigEndian>().unwrap(); // skip.
    let layer = f.read_u16::<BigEndian>().unwrap();
    let alternate_group = f.read_u16::<BigEndian>().unwrap();
    let volume = f.read_u16::<BigEndian>().unwrap() >> 8;

    f.read_u8().unwrap(); // skip.
    let matrix = Matrix{
        a: f.read_i32::<byteorder::LittleEndian>().unwrap(),
        b: f.read_i32::<BigEndian>().unwrap(),
        u: f.read_i32::<BigEndian>().unwrap(),
        c: f.read_i32::<BigEndian>().unwrap(),
        d: f.read_i32::<BigEndian>().unwrap(),
        v: f.read_i32::<BigEndian>().unwrap(),
        x: f.read_i32::<BigEndian>().unwrap(),
        y: f.read_i32::<BigEndian>().unwrap(),
        w: f.read_i32::<BigEndian>().unwrap(),
    };

    let width = f.read_u32::<BigEndian>().unwrap() >> 8;
    let height = f.read_u32::<BigEndian>().unwrap() >> 8;

    // Skip remaining bytes.
    let after =  f.seek(SeekFrom::Current(0)).unwrap();
    let remaining_bytes = (size as u64 - (after - current)) as i64;
    f.seek(SeekFrom::Current(remaining_bytes - HEADER_SIZE as i64)).unwrap();

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

fn parse_edts_box(f: &mut BufReader<File>, _offset: u64, size: u32) -> Result<EdtsBox> {
    let current =  f.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.
    let mut edts = EdtsBox::new();

    let start = 0u64;
    while start < size as u64 {
        // Get box header.
        let header = read_box_header(f, start).unwrap();
        let BoxHeader{ name, size: s, offset } = header;

        let mut b = BMFFBox::new();
        b.head = BoxHeader{
            name: name.try_into().unwrap(),
            size: s as u64,
            offset: offset as u64,
        };

        match b.head.name.as_ref() {
            "elst" => {
                let elst = parse_elst_box(f, 0, s as u32).unwrap();
                edts.elst = Some(elst);
            }
            _ => break
        }
    }

    // Skip remaining bytes.
    let after =  f.seek(SeekFrom::Current(0)).unwrap();
    let remaining_bytes = (size as u64 - (after - current)) as i64;
    f.seek(SeekFrom::Current(remaining_bytes - HEADER_SIZE as i64)).unwrap();
    Ok(edts)
}

fn parse_elst_box(f: &mut BufReader<File>, _offset: u64, size: u32) -> Result<ElstBox> {
    let current = f.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

    let version = f.read_u32::<BigEndian>().unwrap();
    let entry_count = f.read_u32::<BigEndian>().unwrap();

    let mut entries = Vec::new();

    for _i in 0..entry_count {
        let entry = ElstEntry{
            segment_duration: f.read_u32::<BigEndian>().unwrap(),
            media_time: f.read_u32::<BigEndian>().unwrap(),
            media_rate: f.read_u16::<BigEndian>().unwrap(),
            media_rate_fraction: f.read_u16::<BigEndian>().unwrap(),
        };
        entries.push(entry);
    }

    // Skip remaining bytes.
    let after =  f.seek(SeekFrom::Current(0)).unwrap();
    let remaining_bytes = (size as u64 - (after - current)) as i64;
    f.seek(SeekFrom::Current(remaining_bytes - HEADER_SIZE as i64)).unwrap();

    Ok(ElstBox {
        version,
        entry_count,
        entries,
    })
}

fn parse_mdia_box(f: &mut BufReader<File>, _offset: u64, size: u32) -> Result<MdiaBox> {
    let current =  f.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.
    let mut mdia = MdiaBox::new();

    let start = 0u64;
    while start < size as u64 {
        // Get box header.
        let header = read_box_header(f, start).unwrap();
        let BoxHeader{ name, size: s, offset } = header;

        let mut b = BMFFBox::new();
        b.head = BoxHeader{
            name: name.try_into().unwrap(),
            size: s as u64,
            offset: offset as u64,
        };

        match b.head.name.as_ref() {
            "mdhd" => {
                let mdhd = parse_mdhd_box(f, 0, s as u32).unwrap();
                mdia.mdhd = Some(mdhd);
            }
            "hdlr" => {
                let hdlr = parse_hdlr_box(f, 0, s as u32).unwrap();
                mdia.hdlr = Some(hdlr);
            }
            "minf" => {
                let minf = parse_minf_box(f, 0, s as u32).unwrap();
                mdia.minf = Some(minf);
            }
            _ => break
        }
    }

    // Skip remaining bytes.
    let after =  f.seek(SeekFrom::Current(0)).unwrap();
    let remaining_bytes = (size as u64 - (after - current)) as i64;
    f.seek(SeekFrom::Current(remaining_bytes - HEADER_SIZE as i64)).unwrap();
    Ok(mdia)
}

fn parse_mdhd_box(f: &mut BufReader<File>, _offset: u64, size: u32) -> Result<MdhdBox> {
    let current = f.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

    let version = f.read_u8().unwrap();
    let flags_a = f.read_u8().unwrap();
    let flags_b = f.read_u8().unwrap();
    let flags_c = f.read_u8().unwrap();
    let flags = u32::from(flags_a) << 16 | u32::from(flags_b) << 8 | u32::from(flags_c);
    let creation_time = f.read_u32::<BigEndian>().unwrap();
    let modification_time = f.read_u32::<BigEndian>().unwrap();
    let timescale = f.read_u32::<BigEndian>().unwrap();
    let duration = f.read_u32::<BigEndian>().unwrap();
    let language = f.read_u16::<BigEndian>().unwrap();
    let language_string = get_language_string(language);

    // Skip remaining bytes.
    let after =  f.seek(SeekFrom::Current(0)).unwrap();
    let remaining_bytes = (size as u64 - (after - current)) as i64;
    f.seek(SeekFrom::Current(remaining_bytes - HEADER_SIZE as i64)).unwrap();

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

fn parse_hdlr_box(f: &mut BufReader<File>, _offset: u64, size: u32) -> Result<HdlrBox> {
    let current = f.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

    let version = f.read_u8().unwrap();
    let flags_a = f.read_u8().unwrap();
    let flags_b = f.read_u8().unwrap();
    let flags_c = f.read_u8().unwrap();
    let flags = u32::from(flags_a) << 16 | u32::from(flags_b) << 8 | u32::from(flags_c);
    f.read_u32::<BigEndian>().unwrap(); // skip.
    let handler = f.read_u32::<BigEndian>().unwrap();

    let n = f.seek(SeekFrom::Current(12)).unwrap(); // 12 bytes reserved.
    let buf_size = (size as u64 - (n - current)) - HEADER_SIZE as u64;
    let mut buf = vec![0u8; buf_size as usize];
    f.read_exact(&mut buf).unwrap();

    let handler_string = match String::from_utf8(buf) {
        Ok(t) => t,
        _ => String::from("null"),
    };

    // Skip remaining bytes.
    let after = f.seek(SeekFrom::Current(0)).unwrap();
    let remaining_bytes = (size as u64 - (after - current)) as i64;
    f.seek(SeekFrom::Current(remaining_bytes - HEADER_SIZE as i64)).unwrap();

    Ok(HdlrBox {
        version,
        flags,
        handler_type: From::from(handler),
        name: handler_string,
    })
}

fn parse_minf_box(f: &mut BufReader<File>, _offset: u64, size: u32) -> Result<MinfBox> {
    println!("size: {:?}", size);
    let current =  f.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.
    let mut minf = MinfBox::new();

    let mut start = 0u64;
    while start < size as u64 {
        // Get box header.
        let header = read_box_header(f, start).unwrap();
        let BoxHeader{ name, size: s, offset } = header;

        let mut b = BMFFBox::new();
        b.head = BoxHeader{
            name: name.try_into().unwrap(),
            size: s as u64,
            offset: offset as u64,
        };

        match b.head.name.as_ref() {
            "vmhd" => {
                println!("found vmhd");
                let vmhd = parse_vmhd_box(f, 0, s as u32).unwrap();
                minf.vmhd = Some(vmhd);
//                 start = (s as u32 - HEADER_SIZE) as u64;
            }
            "smhd" => {
                println!("found smhd");
////                let vmhd = parse_smhd_box(f, 0, s as u32).unwrap();
////                minf.smhd = Some(vmhd);
                start = (s as u32 - HEADER_SIZE) as u64;
            }
            "dinf" => {
                println!("found dinf");
                start = (s as u32 - HEADER_SIZE) as u64;
            }
            "stbl" => {
                println!("found stbl");
                let stbl = parse_stbl_box(f, 0, s as u32).unwrap();
                minf.stbl = Some(stbl);
                // start = (s as u32 - HEADER_SIZE) as u64;
            }
            _ => break
        }
    }

    // Skip remaining bytes.
    let after =  f.seek(SeekFrom::Current(0)).unwrap();
    let remaining_bytes = (size as u64 - (after - current)) as i64;
    f.seek(SeekFrom::Current(remaining_bytes - HEADER_SIZE as i64)).unwrap();
    Ok(minf)
}

fn parse_vmhd_box(f: &mut BufReader<File>, _offset: u64, size: u32) -> Result<VmhdBox> {
    let current = f.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

    let version = f.read_u8().unwrap();
    let flags_a = f.read_u8().unwrap();
    let flags_b = f.read_u8().unwrap();
    let flags_c = f.read_u8().unwrap();
    let flags = u32::from(flags_a) << 16 | u32::from(flags_b) << 8 | u32::from(flags_c);
//    let flags = f.read_u32::<BigEndian>().unwrap();
    let graphics_mode = f.read_u16::<BigEndian>().unwrap();
    let op_color = f.read_u16::<BigEndian>().unwrap();

    // Skip remaining bytes.
    let after = f.seek(SeekFrom::Current(0)).unwrap();
    let remaining_bytes = (size as u64 - (after - current)) as i64;
    f.seek(SeekFrom::Current(remaining_bytes - HEADER_SIZE as i64)).unwrap();

    Ok(VmhdBox {
        version,
        flags,
        graphics_mode,
        op_color,
    })
}

fn parse_stbl_box(f: &mut BufReader<File>, _offset: u64, size: u32) -> Result<StblBox> {
    println!("stbl size: {:?}", size);
    let current =  f.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.
    let mut stbl = StblBox::new();

    let mut start = 0u64;
    while start < size as u64 {
        // Get box header.
        let header = read_box_header(f, start).unwrap();
        let BoxHeader{ name, size: s, offset } = header;

        let mut b = BMFFBox::new();
        b.head = BoxHeader{
            name: name.try_into().unwrap(),
            size: s as u64,
            offset: offset as u64,
        };

        match b.head.name.as_ref() {
            "stsd" => {
                println!("found stsd: {:?}", s);
//                let stsd = parse_stsd_box(f, 0, s as u32).unwrap();
                start = (s as u32 - HEADER_SIZE) as u64;
            }
            "stts" => {
                let stts = parse_stts_box(f, 0, s as u32).unwrap();
                stbl.stts = Some(stts);
            }
            "stss" => {
                println!("found stss");
                start = (s as u32 - HEADER_SIZE) as u64;
            }
            "ctts" => {
                println!("found ctts");
                start = (s as u32 - HEADER_SIZE) as u64;
            }
            "stsc" => {
                println!("found stsc");
                start = (s as u32 - HEADER_SIZE) as u64;
            }
            "stsz" => {
                println!("found stsz");
                start = (s as u32 - HEADER_SIZE) as u64;
            }
            "stco" => {
                println!("found stco");
                start = (s as u32 - HEADER_SIZE) as u64;
            }
            _ => break
        }
    }

    // Skip remaining bytes.
//    let after =  f.seek(SeekFrom::Current(0)).unwrap();
//    let remaining_bytes = (size as u64 - (after - current)) as i64;
//    f.seek(SeekFrom::Current(remaining_bytes - HEADER_SIZE as i64)).unwrap();
    Ok(stbl)
}

fn parse_stts_box(f: &mut BufReader<File>, _offset: u64, size: u32) -> Result<SttsBox> {
    let current = f.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

    let version = f.read_u8().unwrap();
    let flags_a = f.read_u8().unwrap();
    let flags_b = f.read_u8().unwrap();
    let flags_c = f.read_u8().unwrap();
    let flags = u32::from(flags_a) << 16 | u32::from(flags_b) << 8 | u32::from(flags_c);
    let entry_count = f.read_u32::<BigEndian>().unwrap();
    let mut sample_counts = Vec::new();
    let mut sample_deltas = Vec::new();

    for _i in 0..entry_count {
        let sc = f.read_u32::<BigEndian>().unwrap();
        let sd = f.read_u32::<BigEndian>().unwrap();
        sample_counts.push(sc);
        sample_deltas.push(sd);
    }

    // Skip remaining bytes.
    let after = f.seek(SeekFrom::Current(0)).unwrap();
    let remaining_bytes = (size as u64 - (after - current)) as i64;
    f.seek(SeekFrom::Current(remaining_bytes - HEADER_SIZE as i64)).unwrap();

    Ok(SttsBox {
        version,
        flags,
        entry_count,
        sample_counts,
        sample_deltas,
    })
}

fn parse_stsd_box(f: &mut BufReader<File>, _offset: u64, size: u32) -> Result<StsdBox> {
    let current = f.seek(SeekFrom::Current(0)).unwrap(); // Current cursor position.

    let version = f.read_u8().unwrap();
    let flags_a = f.read_u8().unwrap();
    let flags_b = f.read_u8().unwrap();
    let flags_c = f.read_u8().unwrap();
    let flags = u32::from(flags_a) << 16 | u32::from(flags_b) << 8 | u32::from(flags_c);
    f.read_u32::<BigEndian>().unwrap(); // skip.


    let mut start = 0u64;
    while start < size as u64 {
        // Get box header.
        let header = read_box_header(f, start).unwrap();
        let BoxHeader{ name, size: s, offset } = header;

        let mut b = BMFFBox::new();
        b.head = BoxHeader{
            name: name.try_into().unwrap(),
            size: s as u64,
            offset: offset as u64,
        };

        match b.head.name.as_ref() {
            "avc1" => {
                println!("found avc1");
                start = (s as u32 - HEADER_SIZE) as u64;
            }
            "mp4a" => {
                println!("found mp4a");
                start = (s as u32 - HEADER_SIZE) as u64;
            }
            _ => break
        }
    }

    // Skip remaining bytes.
//    let after = f.seek(SeekFrom::Current(0)).unwrap();
//    let remaining_bytes = (size as u64 - (after - current)) as i64;
//    f.seek(SeekFrom::Current(remaining_bytes - HEADER_SIZE as i64)).unwrap();

    Ok(StsdBox {
        version,
        flags,
    })
}
