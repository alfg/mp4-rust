use std::io::{Read, Seek, SeekFrom};

use crate::atoms::*;
use crate::atoms::{mvhd::MvhdBox, stbl::StblBox, trak::TrakBox};
use crate::{Bytes, Error, Mp4Sample, Result};

#[derive(Debug)]
pub struct Mp4Reader<R> {
    reader: R,
    pub ftyp: FtypBox,
    pub moov: Option<MoovBox>,
    size: u64,

    tracks: Vec<TrackReader>,
}

impl<R: Read + Seek> Mp4Reader<R> {
    pub fn new(reader: R) -> Self {
        Mp4Reader {
            reader,
            ftyp: FtypBox::default(),
            moov: None,
            size: 0,
            tracks: Vec::new(),
        }
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn read(&mut self, size: u64) -> Result<()> {
        let start = self.reader.seek(SeekFrom::Current(0))?;
        let mut current = start;
        while current < size {
            // Get box header.
            let header = BoxHeader::read(&mut self.reader)?;
            let BoxHeader { name, size: s } = header;

            // Match and parse the atom boxes.
            match name {
                BoxType::FtypBox => {
                    let ftyp = FtypBox::read_box(&mut self.reader, s)?;
                    self.ftyp = ftyp;
                }
                BoxType::FreeBox => {
                    skip_box(&mut self.reader, s)?;
                }
                BoxType::MdatBox => {
                    skip_box(&mut self.reader, s)?;
                }
                BoxType::MoovBox => {
                    let moov = MoovBox::read_box(&mut self.reader, s)?;
                    self.moov = Some(moov);
                }
                BoxType::MoofBox => {
                    skip_box(&mut self.reader, s)?;
                }
                _ => {
                    // XXX warn!()
                    skip_box(&mut self.reader, s)?;
                }
            }
            current = self.reader.seek(SeekFrom::Current(0))?;
        }
        if let Some(ref moov) = self.moov {
            for (i, trak) in moov.traks.iter().enumerate() {
                self.tracks.push(TrackReader::new(i as u32 + 1, trak));
            }
        }
        self.size = current - start;
        Ok(())
    }

    pub fn major_brand(&self) -> &FourCC {
        &self.ftyp.major_brand
    }

    pub fn minor_version(&self) -> u32 {
        self.ftyp.minor_version
    }

    pub fn compatible_brands(&self) -> &[FourCC] {
        &self.ftyp.compatible_brands
    }

    fn mvhd(&self) -> Result<&MvhdBox> {
        if let Some(ref moov) = self.moov {
            Ok(&moov.mvhd)
        } else {
            Err(Error::BoxNotFound(BoxType::VmhdBox))
        }
    }

    pub fn duration(&self) -> Result<u64> {
        let mvhd = self.mvhd()?;
        Ok(mvhd.duration)
    }

    pub fn timescale(&self) -> Result<u32> {
        let mvhd = self.mvhd()?;
        Ok(mvhd.timescale)
    }

    pub fn track_count(&self) -> u32 {
        self.tracks.len() as u32
    }

    pub fn tracks(&self) -> &[TrackReader] {
        &self.tracks
    }

    pub fn sample_count(&self, track_id: u32) -> Result<u32> {
        if track_id == 0 {
            return Err(Error::TrakNotFound(track_id));
        }

        if let Some(track) = self.tracks.get(track_id as usize - 1) {
            Ok(track.sample_count())
        } else {
            Err(Error::TrakNotFound(track_id))
        }
    }

    pub fn read_sample(&mut self, track_id: u32, sample_id: u32) -> Result<Option<Mp4Sample>> {
        if track_id == 0 {
            return Err(Error::TrakNotFound(track_id));
        }

        if let Some(track) = self.tracks.get(track_id as usize - 1) {
            track.read_sample(&mut self.reader, sample_id)
        } else {
            Err(Error::TrakNotFound(track_id))
        }
    }
}

#[derive(Debug)]
pub struct TrackReader {
    track_id: u32,
    trak: TrakBox,
}

impl TrackReader {
    pub(crate) fn new(track_id: u32, trak: &TrakBox) -> Self {
        let trak = trak.clone();
        Self { track_id, trak }
    }

    pub fn track_id(&self) -> u32 {
        self.track_id
    }

    fn stbl(&self) -> &StblBox {
        &self.trak.mdia.minf.stbl
    }

    fn stsc_index(&self, sample_id: u32) -> usize {
        let stsc = &self.stbl().stsc;

        for (i, entry) in stsc.entries.iter().enumerate() {
            if sample_id < entry.first_sample {
                assert_ne!(i, 0);
                return i - 1;
            }
        }

        assert_ne!(stsc.entries.len(), 0);
        stsc.entries.len() - 1
    }

    fn chunk_offset(&self, chunk_id: u32) -> Result<u64> {
        let stbl = self.stbl();

        if let Some(ref stco) = stbl.stco {
            if let Some(offset) = stco.entries.get(chunk_id as usize - 1) {
                return Ok(*offset as u64);
            } else {
                return Err(Error::EntryInStblNotFound(
                    self.track_id,
                    BoxType::StcoBox,
                    chunk_id,
                ));
            }
        } else {
            if let Some(ref co64) = stbl.co64 {
                if let Some(offset) = co64.entries.get(chunk_id as usize - 1) {
                    return Ok(*offset);
                } else {
                    return Err(Error::EntryInStblNotFound(
                        self.track_id,
                        BoxType::Co64Box,
                        chunk_id,
                    ));
                }
            }
        }

        assert!(stbl.stco.is_some() || stbl.co64.is_some());
        return Err(Error::Box2NotFound(BoxType::StcoBox, BoxType::Co64Box));
    }

    fn ctts_index(&self, sample_id: u32) -> Result<(usize, u32)> {
        let stbl = self.stbl();

        assert!(stbl.ctts.is_some());
        let ctts = if let Some(ref ctts) = stbl.ctts {
            ctts
        } else {
            return Err(Error::BoxInStblNotFound(self.track_id, BoxType::CttsBox));
        };

        let mut sample_count = 1;
        for (i, entry) in ctts.entries.iter().enumerate() {
            if sample_id <= sample_count + entry.sample_count - 1 {
                return Ok((i, sample_count));
            }
            sample_count += entry.sample_count;
        }

        return Err(Error::EntryInStblNotFound(
            self.track_id,
            BoxType::CttsBox,
            sample_id,
        ));
    }

    pub fn sample_count(&self) -> u32 {
        let stsz = &self.stbl().stsz;
        stsz.sample_sizes.len() as u32
    }

    pub fn sample_size(&self, sample_id: u32) -> Result<u32> {
        let stsz = &self.stbl().stsz;
        if stsz.sample_size > 0 {
            return Ok(stsz.sample_size);
        }
        if let Some(size) = stsz.sample_sizes.get(sample_id as usize - 1) {
            Ok(*size)
        } else {
            return Err(Error::EntryInStblNotFound(
                self.track_id,
                BoxType::StszBox,
                sample_id,
            ));
        }
    }

    pub fn sample_offset(&self, sample_id: u32) -> Result<u64> {
        let stsc_index = self.stsc_index(sample_id);

        let stsc = &self.stbl().stsc;
        let stsc_entry = stsc.entries.get(stsc_index).unwrap();

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

    pub fn sample_time(&self, sample_id: u32) -> Result<(u64, u32)> {
        let stts = &self.stbl().stts;

        let mut sample_count = 1;
        let mut elapsed = 0;

        for entry in stts.entries.iter() {
            if sample_id <= sample_count + entry.sample_count - 1 {
                let start_time =
                    (sample_id - sample_count) as u64 * entry.sample_delta as u64 + elapsed;
                return Ok((start_time, entry.sample_delta));
            }

            sample_count += entry.sample_count;
            elapsed += entry.sample_count as u64 * entry.sample_delta as u64;
        }

        return Err(Error::EntryInStblNotFound(
            self.track_id,
            BoxType::SttsBox,
            sample_id,
        ));
    }

    pub fn sample_rendering_offset(&self, sample_id: u32) -> i32 {
        let stbl = self.stbl();

        if let Some(ref ctts) = stbl.ctts {
            if let Ok((ctts_index, _)) = self.ctts_index(sample_id) {
                let ctts_entry = ctts.entries.get(ctts_index).unwrap();
                return ctts_entry.sample_offset;
            }
        }
        0
    }

    pub fn is_sync_sample(&self, sample_id: u32) -> bool {
        let stbl = self.stbl();

        if let Some(ref stss) = stbl.stss {
            match stss.entries.binary_search(&sample_id) {
                Ok(_) => true,
                Err(_) => false,
            }
        } else {
            true
        }
    }

    pub fn read_sample<R: Read + Seek>(
        &self,
        reader: &mut R,
        sample_id: u32,
    ) -> Result<Option<Mp4Sample>> {
        let sample_size = match self.sample_size(sample_id) {
            Ok(size) => size,
            Err(Error::EntryInStblNotFound(_, _, _)) => return Ok(None),
            Err(err) => return Err(err),
        };
        let sample_offset = self.sample_offset(sample_id).unwrap(); // XXX

        let mut buffer = vec![0x0u8; sample_size as usize];
        reader.seek(SeekFrom::Start(sample_offset))?;
        reader.read_exact(&mut buffer)?;

        let (start_time, duration) = self.sample_time(sample_id).unwrap(); // XXX
        let rendering_offset = self.sample_rendering_offset(sample_id);
        let is_sync = self.is_sync_sample(sample_id);

        Ok(Some(Mp4Sample {
            start_time,
            duration,
            rendering_offset,
            is_sync,
            bytes: Bytes::from(buffer),
        }))
    }
}
