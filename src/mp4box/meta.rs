use std::io::{Read, Seek};

use serde::Serialize;

use crate::mp4box::hdlr::HdlrBox;
use crate::mp4box::ilst::IlstBox;
use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "hdlr")]
#[serde(rename_all = "lowercase")]
pub enum MetaBox {
    Mdir {
        #[serde(skip_serializing_if = "Option::is_none")]
        ilst: Option<IlstBox>,
    },

    #[serde(skip)]
    Unknown {
        #[serde(skip)]
        hdlr: HdlrBox,

        #[serde(skip)]
        data: Vec<u8>,
    },
}

const MDIR: FourCC = FourCC { value: *b"mdir" };

impl Default for MetaBox {
    fn default() -> Self {
        Self::Unknown {
            hdlr: Default::default(),
            data: Default::default(),
        }
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for MetaBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, _) = read_box_header_ext(reader)?;
        if version != 0 {
            return Err(Error::UnsupportedBoxVersion(BoxType::UdtaBox, version));
        }

        let hdlr_header = BoxHeader::read(reader)?;
        if hdlr_header.name != BoxType::HdlrBox {
            return Err(Error::BoxNotFound(BoxType::HdlrBox));
        }
        let hdlr = HdlrBox::read_box(reader, hdlr_header.size)?;

        let mut ilst = None;

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;

        match hdlr.handler_type {
            MDIR => {
                while current < end {
                    // Get box header.
                    let header = BoxHeader::read(reader)?;
                    let BoxHeader { name, size: s } = header;

                    match name {
                        BoxType::IlstBox => {
                            ilst = Some(IlstBox::read_box(reader, s)?);
                        }
                        _ => {
                            // XXX warn!()
                            skip_box(reader, s)?;
                        }
                    }

                    current = reader.seek(SeekFrom::Current(0))?;
                }

                Ok(MetaBox::Mdir { ilst })
            }
            _ => {
                let mut data = vec![0u8; (end - current) as usize];
                reader.read_exact(&mut data)?;

                Ok(MetaBox::Unknown { hdlr, data })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mp4box::BoxHeader;
    use std::io::Cursor;

    #[test]
    fn test_meta_mdir() {
        let src_hdlr = HdlrBox {
            handler_type: MDIR,
            ..Default::default()
        };
        let src_header = BoxHeader::new(
            BoxType::MetaBox,
            HEADER_SIZE + HEADER_EXT_SIZE + src_hdlr.box_size(),
        );

        let mut buf = Vec::new();
        src_header.write(&mut buf).unwrap();
        write_box_header_ext(&mut buf, 0, 0).unwrap();
        src_hdlr.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_header.size as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, src_header.name);
        assert_eq!(header.size, src_header.size);

        let dst_box = MetaBox::read_box(&mut reader, header.size).unwrap();
        assert!(matches!(dst_box, MetaBox::Mdir { ilst: None }));
    }

    #[test]
    fn test_meta_unknown() {
        let src_hdlr = HdlrBox {
            handler_type: FourCC::from(*b"test"),
            ..Default::default()
        };
        let src_data = b"123";
        let src_header = BoxHeader::new(
            BoxType::MetaBox,
            HEADER_SIZE + HEADER_EXT_SIZE + src_hdlr.box_size() + src_data.len() as u64,
        );

        let mut buf = Vec::new();
        src_header.write(&mut buf).unwrap();
        write_box_header_ext(&mut buf, 0, 0).unwrap();
        src_hdlr.write_box(&mut buf).unwrap();

        buf.extend(src_data);

        assert_eq!(buf.len(), src_header.size as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, src_header.name);
        assert_eq!(header.size, src_header.size);

        let dst_box = MetaBox::read_box(&mut reader, header.size).unwrap();
        assert!(
            matches!(dst_box, MetaBox::Unknown { hdlr, data } if data == src_data && hdlr == src_hdlr)
        );
    }
}
