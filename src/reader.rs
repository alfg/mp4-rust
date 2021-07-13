use std::io::{Read, Seek, SeekFrom};
use std::time::Duration;

use crate::mp4box::*;
use crate::*;

#[derive(Debug)]
pub struct Mp4Reader<R> {
    reader: R,
    pub ftyp: FtypBox,
    pub moov: MoovBox,
    pub moofs: Vec<MoofBox>,

    tracks: Vec<Mp4Track>,
    size: u64,
}

impl<R: Read + Seek> Mp4Reader<R> {
    pub fn read_header(mut reader: R, size: u64) -> Result<Self> {
        let start = reader.seek(SeekFrom::Current(0))?;

        let mut ftyp = None;
        let mut moov = None;
        let mut moofs = Vec::new();

        let mut current = start;
        while current < size {
            // Get box header.
            let header = BoxHeader::read(&mut reader)?;
            let BoxHeader { name, size: s } = header;

            // Match and parse the atom boxes.
            match name {
                BoxType::FtypBox => {
                    ftyp = Some(FtypBox::read_box(&mut reader, s)?);
                }
                BoxType::FreeBox => {
                    skip_box(&mut reader, s)?;
                }
                BoxType::MdatBox => {
                    skip_box(&mut reader, s)?;
                }
                BoxType::MoovBox => {
                    moov = Some(MoovBox::read_box(&mut reader, s)?);
                }
                BoxType::MoofBox => {
                    let moof = MoofBox::read_box(&mut reader, s)?;
                    moofs.push(moof);
                }
                _ => {
                    // XXX warn!()
                    skip_box(&mut reader, s)?;
                }
            }
            current = reader.seek(SeekFrom::Current(0))?;
        }

        if ftyp.is_none() {
            return Err(Error::BoxNotFound(BoxType::FtypBox));
        }
        if moov.is_none() {
            return Err(Error::BoxNotFound(BoxType::MoovBox));
        }

        let size = current - start;
        let mut tracks = if let Some(ref moov) = moov {
            let mut tracks = Vec::with_capacity(moov.traks.len());
            for (i, trak) in moov.traks.iter().enumerate() {
                if trak.tkhd.track_id != i as u32 + 1 {
                    return Err(Error::InvalidData("tracks out of order"));
                }
                tracks.push(Mp4Track::from(trak));
            }
            tracks
        } else {
            Vec::new()
        };

        // Update tracks if any fragmented (moof) boxes are found.
        if moofs.len() > 0 {
            let mut default_sample_duration = 0;
            if let Some(ref moov) = moov {
                if let Some(ref mvex) = &moov.mvex {
                    default_sample_duration = mvex.trex.default_sample_duration
                }
            }

            for moof in moofs.iter() {
                for traf in moof.trafs.iter() {
                    let track_id = traf.tfhd.track_id as usize - 1;
                    tracks[track_id].default_sample_duration = default_sample_duration;
                    tracks[track_id].trafs.push(traf.clone());
                }
            }
        }

        Ok(Mp4Reader {
            reader,
            ftyp: ftyp.unwrap(),
            moov: moov.unwrap(),
            moofs,
            size,
            tracks,
        })
    }

    pub fn size(&self) -> u64 {
        self.size
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

    pub fn duration(&self) -> Duration {
        Duration::from_millis(self.moov.mvhd.duration * 1000 / self.moov.mvhd.timescale as u64)
    }

    pub fn timescale(&self) -> u32 {
        self.moov.mvhd.timescale
    }

    pub fn is_fragmented(&self) -> bool {
        self.moofs.len() != 0
    }

    pub fn tracks(&self) -> &[Mp4Track] {
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

        if let Some(ref track) = self.tracks.get(track_id as usize - 1) {
            track.read_sample(&mut self.reader, sample_id)
        } else {
            Err(Error::TrakNotFound(track_id))
        }
    }
}
