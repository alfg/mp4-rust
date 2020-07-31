use std::io::{Seek, SeekFrom, Read, Write};

use crate::*;
use crate::atoms::*;
use crate::atoms::{
    tkhd::TkhdBox,
    edts::EdtsBox,
    mdia::MdiaBox,
    stbl::StblBox,
    stts::SttsBox,
    stsc::StscBox,
    stsz::StszBox,
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
        if let Some(ref mdia) = self.mdia {
            if let Some(ref minf) = mdia.minf {
                if let Some(ref stbl) = minf.stbl {
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

    fn stts(&self) -> Result<&SttsBox> {
        let stbl = self.stbl()?;

        if let Some(ref stts) = stbl.stts {
            Ok(stts)
        } else {
            Err(Error::BoxInStblNotFound(self.id, BoxType::SttsBox))
        }
    }

    fn stsc(&self) -> Result<&StscBox> {
        let stbl = self.stbl()?;

        if let Some(ref stsc) = stbl.stsc {
            Ok(stsc)
        } else {
            Err(Error::BoxInStblNotFound(self.id, BoxType::StscBox))
        }
    }

    fn stsz(&self) -> Result<&StszBox> {
        let stbl = self.stbl()?;

        if let Some(ref stsz) = stbl.stsz {
            Ok(stsz)
        } else {
            Err(Error::BoxInStblNotFound(self.id, BoxType::StszBox))
        }
    }

    fn stsc_index(&self, sample_id: u32) -> Result<usize> {
        let stsc = self.stsc()?;

        for (i, entry) in stsc.entries.iter().enumerate() {
            if sample_id < entry.first_sample {
                assert_ne!(i, 0);
                return Ok(i - 1);
            }
        }

        assert_ne!(stsc.entries.len(), 0);
        Ok(stsc.entries.len() - 1)
    }

    fn chunk_offset(&self, chunk_id: u32) -> Result<u64> {
        let stbl = self.stbl()?;

        if let Some(ref stco) = stbl.stco {
            if let Some(offset) = stco.entries.get(chunk_id as usize - 1) {
                return Ok(*offset as u64);
            } else {
                return Err(Error::EntryInStblNotFound(self.id, BoxType::StcoBox,
                                                      chunk_id));
            }
        } else {
            if let Some(ref co64) = stbl.co64 {
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

    fn ctts_index(&self, sample_id: u32) -> Result<(usize, u32)> {
        let stbl = self.stbl()?;

        let ctts = if let Some(ref ctts) = stbl.ctts {
            ctts
        } else {
            return Err(Error::BoxInStblNotFound(self.id, BoxType::CttsBox));
        };

        let mut sample_count = 1;
        for (i, entry) in ctts.entries.iter().enumerate() {
            if sample_id <= sample_count + entry.sample_count -1 {
                return Ok((i, sample_count))
            }
            sample_count += entry.sample_count;
        }

        return Err(Error::EntryInStblNotFound(self.id, BoxType::CttsBox, sample_id));
    }

    pub fn sample_count(&self) -> Result<u32> {
        let stsz = self.stsz()?;
        Ok(stsz.sample_sizes.len() as u32)
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
        let stsc_index = self.stsc_index(sample_id)?;

        let stsc = self.stsc()?;
        let stsc_entry = stsc.entries.get(stsc_index).unwrap();

        let first_chunk = stsc_entry.first_chunk;
        let first_sample = stsc_entry.first_sample;
        let samples_per_chunk = stsc_entry.samples_per_chunk;

        let chunk_id = first_chunk + (sample_id - first_sample) / samples_per_chunk;

        let chunk_offset = self.chunk_offset(chunk_id)?;

        let first_sample_in_chunk = sample_id - (sample_id - first_sample)
            % samples_per_chunk;

        let mut sample_offset = 0;
        for i in first_sample_in_chunk..sample_id {
            sample_offset += self.sample_size(i)?;
        }

        Ok(chunk_offset + sample_offset as u64)
    }

    pub fn sample_time(&self, sample_id: u32) -> Result<(u64, u32)> {
        let stts = self.stts()?;

        let mut sample_count = 1;
        let mut elapsed = 0;

        for entry in stts.entries.iter() {
            if sample_id <= sample_count + entry.sample_count - 1 {
                let start_time = (sample_id - sample_count) as u64
                    * entry.sample_delta as u64 + elapsed;
                return Ok((start_time, entry.sample_delta));
            }

            sample_count += entry.sample_count;
            elapsed += entry.sample_count as u64 * entry.sample_delta as u64;
        }

        return Err(Error::EntryInStblNotFound(self.id, BoxType::SttsBox, sample_id));
    }

    pub fn sample_rendering_offset(&self, sample_id: u32) -> Result<i32> {
        let stbl = self.stbl()?;

        if let Some(ref ctts) = stbl.ctts {
            let (ctts_index, _) = self.ctts_index(sample_id)?;
            let ctts_entry = ctts.entries.get(ctts_index).unwrap();
            Ok(ctts_entry.sample_offset)
        } else {
            Ok(0)
        }
    }

    pub fn is_sync_sample(&self, sample_id: u32) -> Result<bool> {
        let stbl = self.stbl()?;

        if let Some(ref stss) = stbl.stss {
            match stss.entries.binary_search(&sample_id) {
                Ok(_) => Ok(true),
                Err(_) => Ok(false)
            }
        } else {
            Ok(true)
        }
    }

    pub fn read_sample<R: Read + Seek>(
        &self,
        reader: &mut R,
        sample_id: u32,
    ) -> Result<Option<Mp4Sample>> {
        let sample_offset = match self.sample_offset(sample_id) {
            Ok(offset) => offset,
            Err(Error::EntryInStblNotFound(_,_,_)) => return Ok(None),
            Err(err) => return Err(err)
        };
        let sample_size = self.sample_size(sample_id)?;

        let mut buffer = vec![0x0u8; sample_size as usize];
        reader.seek(SeekFrom::Start(sample_offset))?;
        reader.read_exact(&mut buffer)?;

        let (start_time, duration) = self.sample_time(sample_id)?;
        let rendering_offset = self.sample_rendering_offset(sample_id)?;
        let is_sync = self.is_sync_sample(sample_id)?;

        Ok(Some(Mp4Sample {
            start_time,
            duration,
            rendering_offset,
            is_sync,
            bytes: Bytes::from(buffer),
        }))
    }
}

impl Mp4Box for TrakBox {
    fn box_type() -> BoxType {
        BoxType::TrakBox
    }

    fn box_size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        if let Some(ref tkhd) = self.tkhd {
            size += tkhd.box_size();
        }
        if let Some(ref edts) = self.edts {
            size += edts.box_size();
        }
        if let Some(ref mdia) = self.mdia {
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

        if let Some(ref tkhd) = self.tkhd {
            tkhd.write_box(writer)?;
        }
        if let Some(ref edts) = self.edts {
            edts.write_box(writer)?;
        }
        if let Some(ref mdia) = self.mdia {
            mdia.write_box(writer)?;
        }

        Ok(size)
    }
}
