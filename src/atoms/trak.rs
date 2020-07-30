use std::io::{Seek, SeekFrom, Read, Write};

use crate::*;
use crate::atoms::*;
use crate::atoms::{
    tkhd::TkhdBox,
    edts::EdtsBox,
    mdia::MdiaBox,
    stbl::StblBox,
    // stsd::StsdBox,
    // stts::SttsBox,
    // stss::StssBox,
    stsc::StscBox,
    stsz::StszBox,
    // stco::StcoBox,
    // co64::Co64Box,
};


#[derive(Debug, Default)]
pub struct TrakBox {
    pub id: u32,

    pub tkhd: Option<TkhdBox>,
    pub edts: Option<EdtsBox>,
    pub mdia: Option<MdiaBox>,
}

impl TrakBox {
    pub(crate) fn new() -> TrakBox {
        Default::default()
    }

    fn stbl(&self) -> Result<&StblBox> {
        if let Some(mdia) = &self.mdia {
            if let Some(minf) = &mdia.minf {
                if let Some(stbl) = &minf.stbl {
                    Ok(stbl)
                } else {
                    Err(Error::BoxInTrakNotFound(self.id, BoxType::StblBox))
                }
            } else {
                Err(Error::BoxInTrakNotFound(self.id, BoxType::MinfBox))
            }
        } else {
            Err(Error::BoxInTrakNotFound(self.id, BoxType::MdiaBox))
        }
    }

    fn stsc(&self) -> Result<&StscBox> {
        let stbl = self.stbl()?;

        if let Some(stsc) = &stbl.stsc {
            Ok(stsc)
        } else {
            Err(Error::BoxInStblNotFound(self.id, BoxType::StscBox))
        }
    }

    fn stsz(&self) -> Result<&StszBox> {
        let stbl = self.stbl()?;

        if let Some(stsz) = &stbl.stsz {
            Ok(stsz)
        } else {
            Err(Error::BoxInStblNotFound(self.id, BoxType::StszBox))
        }
    }

    fn sample_to_stsc_index(&self, sample_id: u32) -> Result<usize> {
        let stsc = self.stsc()?;

        for (i, entry) in stsc.entries.iter().enumerate() {
            if sample_id < entry.first_sample {
                assert_eq!(i, 0);
                return Ok(i - 1);
            }
        }

        assert_eq!(stsc.entries.len(), 0);
        Ok(stsc.entries.len() - 1)
    }

    fn chunk_offset(&self, chunk_id: u32) -> Result<u64> {
        let stbl = self.stbl()?;

        if let Some(stco) = &stbl.stco {
            if let Some(offset) = stco.entries.get(chunk_id as usize - 1) {
                return Ok(*offset as u64);
            } else {
                return Err(Error::EntryInStblNotFound(self.id, BoxType::StcoBox,
                                                      chunk_id));
            }
        } else {
            if let Some(co64) = &stbl.co64 {
                if let Some(offset) = co64.entries.get(chunk_id as usize - 1) {
                    return Ok(*offset);
                } else {
                    return Err(Error::EntryInStblNotFound(self.id, BoxType::Co64Box,
                                                          chunk_id));
                }
            } else {
                // XXX BoxType::StcoBox & BoxType::Co64Box
                Err(Error::BoxInStblNotFound(self.id, BoxType::Co64Box))
            }
        }
    }

    pub fn sample_size(&self, sample_id: u32) -> Result<u32> {
        let stsz = self.stsz()?;
        if stsz.sample_size > 0 {
            return Ok(stsz.sample_size);
        }
        if let Some(size) = stsz.sample_sizes.get(sample_id as usize - 1) {
            Ok(*size)
        } else {
            return Err(Error::EntryInStblNotFound(self.id, BoxType::StszBox, sample_id));
        }
    }

    pub fn sample_offset(&self, sample_id: u32) -> Result<u64> {
        let stsc_index = self.sample_to_stsc_index(sample_id)?;

        let stsc = self.stsc()?;
        let stsc_entry = if let Some(entry) = stsc.entries.get(stsc_index) {
            entry
        } else {
            return Err(Error::EntryInStblNotFound(self.id, BoxType::StscBox,
                                                  stsc_index as u32 + 1));
        };

        let first_chunk = stsc_entry.first_chunk;
        let first_sample = stsc_entry.first_sample;
        let samples_per_chunk = stsc_entry.samples_per_chunk;

        let chunk_id = first_chunk + (sample_id - first_sample) / samples_per_chunk;

        let chunk_offset = self.chunk_offset(chunk_id)?;

        let first_sample_in_chunk = sample_id - (sample_id - first_sample) % samples_per_chunk;

        let mut sample_offset = 0;
        for i in first_sample_in_chunk..sample_id {
            sample_offset += self.sample_size(i)?;
        }

        Ok(chunk_offset + sample_offset as u64)
    }
}

impl Mp4Box for TrakBox {
    fn box_type() -> BoxType {
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

impl<R: Read + Seek> ReadBox<&mut R> for TrakBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = get_box_start(reader)?;

        let mut trak = TrakBox::new();

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader{ name, size: s } = header;

            match name {
                BoxType::TkhdBox => {
                    let tkhd = TkhdBox::read_box(reader, s)?;
                    trak.tkhd = Some(tkhd);
                }
                BoxType::EdtsBox => {
                    let edts = EdtsBox::read_box(reader, s)?;
                    trak.edts = Some(edts);
                }
                BoxType::MdiaBox => {
                    let mdia = MdiaBox::read_box(reader, s)?;
                    trak.mdia = Some(mdia);
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }

            current = reader.seek(SeekFrom::Current(0))?;
        }

        skip_read_to(reader, start + size)?;

        Ok(trak)
    }
}

impl<W: Write> WriteBox<&mut W> for TrakBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(Self::box_type(), size).write(writer)?;

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
