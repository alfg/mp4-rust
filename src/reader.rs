use std::collections::HashMap;
use std::io::{Read, Seek};
use std::time::Duration;

use crate::meta::MetaBox;
use crate::*;

#[derive(Debug)]
pub struct Mp4Header {
    pub ftyp: FtypBox,
    pub moov: MoovBox,
    pub moofs: Vec<MoofBox>,
    pub emsgs: Vec<EmsgBox>,

    tracks: HashMap<u32, Mp4Track>,
    size: u64,
}

impl Mp4Header {
    pub fn read<R: Read + Seek>(reader: &mut R, size: u64) -> Result<Self> {
        let start = reader.stream_position()?;

        let mut ftyp = None;
        let mut moov = None;
        let mut moofs = Vec::new();
        let mut moof_offsets = Vec::new();
        let mut emsgs = Vec::new();

        let mut current = start;
        while current < size {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader { name, size: s } = header;
            if s > size {
                return Err(Error::InvalidData(
                    "file contains a box with a larger size than it",
                ));
            }

            // Break if size zero BoxHeader, which can result in dead-loop.
            if s == 0 {
                break;
            }

            // Match and parse the atom boxes.
            match name {
                BoxType::FtypBox => {
                    ftyp = Some(FtypBox::read_box(reader, s)?);
                }
                BoxType::FreeBox => {
                    skip_box(reader, s)?;
                }
                BoxType::MdatBox => {
                    skip_box(reader, s)?;
                }
                BoxType::MoovBox => {
                    moov = Some(MoovBox::read_box(reader, s)?);
                }
                BoxType::MoofBox => {
                    let moof_offset = reader.stream_position()? - 8;
                    let moof = MoofBox::read_box(reader, s)?;
                    moofs.push(moof);
                    moof_offsets.push(moof_offset);
                }
                BoxType::EmsgBox => {
                    let emsg = EmsgBox::read_box(reader, s)?;
                    emsgs.push(emsg);
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }
            current = reader.stream_position()?;
        }

        if ftyp.is_none() {
            return Err(Error::BoxNotFound(BoxType::FtypBox));
        }
        if moov.is_none() {
            return Err(Error::BoxNotFound(BoxType::MoovBox));
        }

        let size = current - start;
        let mut tracks = if let Some(ref moov) = moov {
            if moov.traks.iter().any(|trak| trak.tkhd.track_id == 0) {
                return Err(Error::InvalidData("illegal track id 0"));
            }
            moov.traks
                .iter()
                .map(|trak| (trak.tkhd.track_id, Mp4Track::from(trak)))
                .collect()
        } else {
            HashMap::new()
        };

        // Update tracks if any fragmented (moof) boxes are found.
        if !moofs.is_empty() {
            let mut default_sample_duration = 0;
            if let Some(ref moov) = moov {
                if let Some(ref mvex) = &moov.mvex {
                    default_sample_duration = mvex.trex.default_sample_duration
                }
            }

            for (moof, moof_offset) in moofs.iter().zip(moof_offsets) {
                for traf in moof.trafs.iter() {
                    let track_id = traf.tfhd.track_id;
                    if let Some(track) = tracks.get_mut(&track_id) {
                        track.default_sample_duration = default_sample_duration;
                        track.moof_offsets.push(moof_offset);
                        track.trafs.push(traf.clone())
                    } else {
                        return Err(Error::TrakNotFound(track_id));
                    }
                }
            }
        }

        Ok(Mp4Header {
            ftyp: ftyp.unwrap(),
            moov: moov.unwrap(),
            moofs,
            emsgs,
            size,
            tracks,
        })
    }

    pub fn read_fragment<R: Read + Seek>(&self, reader: &mut R, size: u64) -> Result<Self> {
        let start = reader.stream_position()?;

        let mut moofs = Vec::new();
        let mut moof_offsets = Vec::new();

        let mut current = start;
        while current < size {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader { name, size: s } = header;
            if s > size {
                return Err(Error::InvalidData(
                    "file contains a box with a larger size than it",
                ));
            }

            // Break if size zero BoxHeader, which can result in dead-loop.
            if s == 0 {
                break;
            }

            // Match and parse the atom boxes.
            match name {
                BoxType::MdatBox => {
                    skip_box(reader, s)?;
                }
                BoxType::MoofBox => {
                    let moof_offset = reader.stream_position()? - 8;
                    let moof = MoofBox::read_box(reader, s)?;
                    moofs.push(moof);
                    moof_offsets.push(moof_offset);
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }
            current = reader.stream_position()?;
        }

        if moofs.is_empty() {
            return Err(Error::BoxNotFound(BoxType::MoofBox));
        }

        let size = current - start;
        let mut tracks: HashMap<u32, Mp4Track> = self
            .moov
            .traks
            .iter()
            .map(|trak| (trak.tkhd.track_id, Mp4Track::from(trak)))
            .collect();

        let mut default_sample_duration = 0;
        if let Some(ref mvex) = &self.moov.mvex {
            default_sample_duration = mvex.trex.default_sample_duration
        }

        for (moof, moof_offset) in moofs.iter().zip(moof_offsets) {
            for traf in moof.trafs.iter() {
                let track_id = traf.tfhd.track_id;
                if let Some(track) = tracks.get_mut(&track_id) {
                    track.default_sample_duration = default_sample_duration;
                    track.moof_offsets.push(moof_offset);
                    track.trafs.push(traf.clone())
                } else {
                    return Err(Error::TrakNotFound(track_id));
                }
            }
        }

        Ok(Mp4Header {
            ftyp: self.ftyp.clone(),
            moov: self.moov.clone(),
            moofs,
            emsgs: Vec::new(),
            tracks,
            size,
        })
    }

    #[inline]
    pub fn size(&self) -> u64 {
        self.size
    }

    #[inline]
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
        !self.moofs.is_empty()
    }

    pub fn tracks(&self) -> &HashMap<u32, Mp4Track> {
        &self.tracks
    }

    pub fn sample_count(&self, track_id: u32) -> Result<u32> {
        if let Some(track) = self.tracks.get(&track_id) {
            Ok(track.sample_count())
        } else {
            Err(Error::TrakNotFound(track_id))
        }
    }

    pub fn read_sample<R: Read + Seek>(
        &mut self,
        reader: &mut R,
        track_id: u32,
        sample_id: u32,
    ) -> Result<Option<Mp4Sample>> {
        if let Some(track) = self.tracks.get(&track_id) {
            track.read_sample(reader, sample_id)
        } else {
            Err(Error::TrakNotFound(track_id))
        }
    }

    pub fn sample_offset(&mut self, track_id: u32, sample_id: u32) -> Result<u64> {
        if let Some(track) = self.tracks.get(&track_id) {
            track.sample_offset(sample_id)
        } else {
            Err(Error::TrakNotFound(track_id))
        }
    }

    pub fn metadata(&self) -> impl Metadata<'_> {
        self.moov.udta.as_ref().and_then(|udta| {
            udta.meta.as_ref().and_then(|meta| match meta {
                MetaBox::Mdir { ilst } => ilst.as_ref(),
                _ => None,
            })
        })
    }
}

#[derive(Debug)]
pub struct Mp4Reader<R> {
    reader: R,
    pub header: Mp4Header,
}

impl<R: Read + Seek> Mp4Reader<R> {
    pub fn from_reader(reader: R, header: Mp4Header) -> Self {
        Self { reader, header }
    }

    pub fn read_header(mut reader: R, size: u64) -> Result<Self> {
        Ok(Mp4Reader {
            header: Mp4Header::read(&mut reader, size)?,
            reader,
        })
    }

    pub fn read_fragment_header<FR: Read + Seek>(
        &self,
        mut reader: FR,
        size: u64,
    ) -> Result<Mp4Reader<FR>> {
        Ok(Mp4Reader {
            header: self.header.read_fragment(&mut reader, size)?,
            reader,
        })
    }

    pub fn size(&self) -> u64 {
        self.header.size()
    }

    pub fn major_brand(&self) -> &FourCC {
        self.header.major_brand()
    }

    pub fn minor_version(&self) -> u32 {
        self.header.minor_version()
    }

    pub fn compatible_brands(&self) -> &[FourCC] {
        self.header.compatible_brands()
    }

    pub fn duration(&self) -> Duration {
        self.header.duration()
    }

    pub fn timescale(&self) -> u32 {
        self.header.timescale()
    }

    pub fn is_fragmented(&self) -> bool {
        self.header.is_fragmented()
    }

    pub fn tracks(&self) -> &HashMap<u32, Mp4Track> {
        self.header.tracks()
    }

    pub fn sample_count(&self, track_id: u32) -> Result<u32> {
        self.header.sample_count(track_id)
    }

    pub fn read_sample(&mut self, track_id: u32, sample_id: u32) -> Result<Option<Mp4Sample>> {
        self.header
            .read_sample(&mut self.reader, track_id, sample_id)
    }

    pub fn sample_offset(&mut self, track_id: u32, sample_id: u32) -> Result<u64> {
        self.header.sample_offset(track_id, sample_id)
    }
}

impl<R> Mp4Reader<R> {
    pub fn metadata(&self) -> impl Metadata<'_> {
        self.header.metadata()
    }
}
