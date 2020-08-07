use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FtypBox {
    pub major_brand: FourCC,
    pub minor_version: u32,
    pub compatible_brands: Vec<FourCC>,
}

impl Mp4Box for FtypBox {
    fn box_type() -> BoxType {
        BoxType::FtypBox
    }

    fn box_size(&self) -> u64 {
        HEADER_SIZE + 8 + (4 * self.compatible_brands.len() as u64)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for FtypBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let major = reader.read_u32::<BigEndian>()?;
        let minor = reader.read_u32::<BigEndian>()?;
        if size % 4 != 0 {
            return Err(Error::InvalidData("invalid ftyp size"));
        }
        let brand_count = (size - 16) / 4; // header + major + minor

        let mut brands = Vec::new();
        for _ in 0..brand_count {
            let b = reader.read_u32::<BigEndian>()?;
            brands.push(From::from(b));
        }

        skip_bytes_to(reader, start + size)?;

        Ok(FtypBox {
            major_brand: From::from(major),
            minor_version: minor,
            compatible_brands: brands,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for FtypBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(Self::box_type(), size).write(writer)?;

        writer.write_u32::<BigEndian>((&self.major_brand).into())?;
        writer.write_u32::<BigEndian>(self.minor_version)?;
        for b in self.compatible_brands.iter() {
            writer.write_u32::<BigEndian>(b.into())?;
        }
        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mp4box::BoxHeader;
    use std::io::Cursor;

    #[test]
    fn test_ftyp() {
        let src_box = FtypBox {
            major_brand: FourCC {
                value: String::from("isom"),
            },
            minor_version: 0,
            compatible_brands: vec![
                FourCC {
                    value: String::from("isom"),
                },
                FourCC {
                    value: String::from("iso2"),
                },
                FourCC {
                    value: String::from("avc1"),
                },
                FourCC {
                    value: String::from("mp41"),
                },
            ],
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::FtypBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = FtypBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
