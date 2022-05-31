use bytes::BytesMut;
use std::cmp;
use std::convert::TryFrom;
use std::io::{Read, Seek, SeekFrom, Write};
use std::time::Duration;

use crate::mp4box::traf::TrafBox;
use crate::mp4box::trak::TrakBox;
use crate::mp4box::{
    avc1::Avc1Box, co64::Co64Box, ctts::CttsBox, ctts::CttsEntry, hev1::Hev1Box, mp4a::Mp4aBox,
    smhd::SmhdBox, stco::StcoBox, stsc::StscEntry, stss::StssBox, stts::SttsEntry, tx3g::Tx3gBox,
    vmhd::VmhdBox, vp09::Vp09Box,
};
use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct TrackConfig {
    pub track_type: TrackType,
    pub timescale: u32,
    pub language: String,
    pub media_conf: MediaConfig,
}

impl From<MediaConfig> for TrackConfig {
    fn from(media_conf: MediaConfig) -> Self {
        match media_conf {
            MediaConfig::AvcConfig(avc_conf) => Self::from(avc_conf),
            MediaConfig::HevcConfig(hevc_conf) => Self::from(hevc_conf),
            MediaConfig::AacConfig(aac_conf) => Self::from(aac_conf),
            MediaConfig::TtxtConfig(ttxt_conf) => Self::from(ttxt_conf),
            MediaConfig::Vp9Config(vp9_config) => Self::from(vp9_config),
        }
    }
}

impl From<AvcConfig> for TrackConfig {
    fn from(avc_conf: AvcConfig) -> Self {
        Self {
            track_type: TrackType::Video,
            timescale: 1000,               // XXX
            language: String::from("und"), // XXX
            media_conf: MediaConfig::AvcConfig(avc_conf),
        }
    }
}

impl From<HevcConfig> for TrackConfig {
    fn from(hevc_conf: HevcConfig) -> Self {
        Self {
            track_type: TrackType::Video,
            timescale: 1000,               // XXX
            language: String::from("und"), // XXX
            media_conf: MediaConfig::HevcConfig(hevc_conf),
        }
    }
}

impl From<AacConfig> for TrackConfig {
    fn from(aac_conf: AacConfig) -> Self {
        Self {
            track_type: TrackType::Audio,
            timescale: 1000,               // XXX
            language: String::from("und"), // XXX
            media_conf: MediaConfig::AacConfig(aac_conf),
        }
    }
}

impl From<TtxtConfig> for TrackConfig {
    fn from(txtt_conf: TtxtConfig) -> Self {
        Self {
            track_type: TrackType::Subtitle,
            timescale: 1000,               // XXX
            language: String::from("und"), // XXX
            media_conf: MediaConfig::TtxtConfig(txtt_conf),
        }
    }
}

impl From<Vp9Config> for TrackConfig {
    fn from(vp9_conf: Vp9Config) -> Self {
        Self {
            track_type: TrackType::Video,
            timescale: 1000,               // XXX
            language: String::from("und"), // XXX
            media_conf: MediaConfig::Vp9Config(vp9_conf),
        }
    }
}

#[derive(Debug)]
pub struct Mp4Track {
    pub trak: TrakBox,
    pub trafs: Vec<TrafBox>,

    // Fragmented Tracks Defaults.
    pub default_sample_duration: u32,
}

impl Mp4Track {
    pub(crate) fn from(trak: &TrakBox) -> Self {
        let trak = trak.clone();
        Self {
            trak,
            trafs: Vec::new(),
            default_sample_duration: 0,
        }
    }

    pub fn track_id(&self) -> u32 {
        self.trak.tkhd.track_id
    }

    pub fn track_type(&self) -> Result<TrackType> {
        TrackType::try_from(&self.trak.mdia.hdlr.handler_type)
    }

    pub fn media_type(&self) -> Result<MediaType> {
        if self.trak.mdia.minf.stbl.stsd.avc1.is_some() {
            Ok(MediaType::H264)
        } else if self.trak.mdia.minf.stbl.stsd.hev1.is_some() {
            Ok(MediaType::H265)
        } else if self.trak.mdia.minf.stbl.stsd.vp09.is_some() {
            Ok(MediaType::VP9)
        } else if self.trak.mdia.minf.stbl.stsd.mp4a.is_some() {
            Ok(MediaType::AAC)
        } else if self.trak.mdia.minf.stbl.stsd.tx3g.is_some() {
            Ok(MediaType::TTXT)
        } else {
            Err(Error::InvalidData("unsupported media type"))
        }
    }

    pub fn box_type(&self) -> Result<FourCC> {
        if self.trak.mdia.minf.stbl.stsd.avc1.is_some() {
            Ok(FourCC::from(BoxType::Avc1Box))
        } else if self.trak.mdia.minf.stbl.stsd.hev1.is_some() {
            Ok(FourCC::from(BoxType::Hev1Box))
        } else if self.trak.mdia.minf.stbl.stsd.vp09.is_some() {
            Ok(FourCC::from(BoxType::Vp09Box))
        } else if self.trak.mdia.minf.stbl.stsd.mp4a.is_some() {
            Ok(FourCC::from(BoxType::Mp4aBox))
        } else if self.trak.mdia.minf.stbl.stsd.tx3g.is_some() {
            Ok(FourCC::from(BoxType::Tx3gBox))
        } else {
            Err(Error::InvalidData("unsupported sample entry box"))
        }
    }

    pub fn width(&self) -> u16 {
        if let Some(ref avc1) = self.trak.mdia.minf.stbl.stsd.avc1 {
            avc1.width
        } else {
            self.trak.tkhd.width.value()
        }
    }

    pub fn height(&self) -> u16 {
        if let Some(ref avc1) = self.trak.mdia.minf.stbl.stsd.avc1 {
            avc1.height
        } else {
            self.trak.tkhd.height.value()
        }
    }

    pub fn frame_rate(&self) -> f64 {
        let dur_msec = self.duration().as_millis() as u64;
        if dur_msec > 0 {
            ((self.sample_count() as u64 * 1000) / dur_msec) as f64
        } else {
            0.0
        }
    }

    pub fn sample_freq_index(&self) -> Result<SampleFreqIndex> {
        if let Some(ref mp4a) = self.trak.mdia.minf.stbl.stsd.mp4a {
            if let Some(ref esds) = mp4a.esds {
                SampleFreqIndex::try_from(esds.es_desc.dec_config.dec_specific.freq_index)
            } else {
                Err(Error::BoxInStblNotFound(self.track_id(), BoxType::EsdsBox))
            }
        } else {
            Err(Error::BoxInStblNotFound(self.track_id(), BoxType::Mp4aBox))
        }
    }

    pub fn channel_config(&self) -> Result<ChannelConfig> {
        if let Some(ref mp4a) = self.trak.mdia.minf.stbl.stsd.mp4a {
            if let Some(ref esds) = mp4a.esds {
                ChannelConfig::try_from(esds.es_desc.dec_config.dec_specific.chan_conf)
            } else {
                Err(Error::BoxInStblNotFound(self.track_id(), BoxType::EsdsBox))
            }
        } else {
            Err(Error::BoxInStblNotFound(self.track_id(), BoxType::Mp4aBox))
        }
    }

    pub fn language(&self) -> &str {
        &self.trak.mdia.mdhd.language
    }

    pub fn timescale(&self) -> u32 {
        self.trak.mdia.mdhd.timescale
    }

    pub fn duration(&self) -> Duration {
        Duration::from_micros(
            self.trak.mdia.mdhd.duration * 1_000_000 / self.trak.mdia.mdhd.timescale as u64,
        )
    }

    pub fn bitrate(&self) -> u32 {
        if let Some(ref mp4a) = self.trak.mdia.minf.stbl.stsd.mp4a {
            if let Some(ref esds) = mp4a.esds {
                esds.es_desc.dec_config.avg_bitrate
            } else {
                0
            }
            // mp4a.esds.es_desc.dec_config.avg_bitrate
        } else {
            let dur_sec = self.duration().as_secs();
            if dur_sec > 0 {
                let bitrate = self.total_sample_size() * 8 / dur_sec;
                bitrate as u32
            } else {
                0
            }
        }
    }

    pub fn sample_count(&self) -> u32 {
        if !self.trafs.is_empty() {
            let mut sample_count = 0u32;
            for traf in self.trafs.iter() {
                if let Some(ref trun) = traf.trun {
                    sample_count += trun.sample_count;
                }
            }
            sample_count
        } else {
            self.trak.mdia.minf.stbl.stsz.sample_count
        }
    }

    pub fn video_profile(&self) -> Result<AvcProfile> {
        if let Some(ref avc1) = self.trak.mdia.minf.stbl.stsd.avc1 {
            AvcProfile::try_from((
                avc1.avcc.avc_profile_indication,
                avc1.avcc.profile_compatibility,
            ))
        } else {
            Err(Error::BoxInStblNotFound(self.track_id(), BoxType::Avc1Box))
        }
    }

    pub fn sequence_parameter_set(&self) -> Result<&[u8]> {
        if let Some(ref avc1) = self.trak.mdia.minf.stbl.stsd.avc1 {
            match avc1.avcc.sequence_parameter_sets.get(0) {
                Some(nal) => Ok(nal.bytes.as_ref()),
                None => Err(Error::EntryInStblNotFound(
                    self.track_id(),
                    BoxType::AvcCBox,
                    0,
                )),
            }
        } else {
            Err(Error::BoxInStblNotFound(self.track_id(), BoxType::Avc1Box))
        }
    }

    pub fn picture_parameter_set(&self) -> Result<&[u8]> {
        if let Some(ref avc1) = self.trak.mdia.minf.stbl.stsd.avc1 {
            match avc1.avcc.picture_parameter_sets.get(0) {
                Some(nal) => Ok(nal.bytes.as_ref()),
                None => Err(Error::EntryInStblNotFound(
                    self.track_id(),
                    BoxType::AvcCBox,
                    0,
                )),
            }
        } else {
            Err(Error::BoxInStblNotFound(self.track_id(), BoxType::Avc1Box))
        }
    }

    pub fn audio_profile(&self) -> Result<AudioObjectType> {
        if let Some(ref mp4a) = self.trak.mdia.minf.stbl.stsd.mp4a {
            if let Some(ref esds) = mp4a.esds {
                AudioObjectType::try_from(esds.es_desc.dec_config.dec_specific.profile)
            } else {
                Err(Error::BoxInStblNotFound(self.track_id(), BoxType::EsdsBox))
            }
        } else {
            Err(Error::BoxInStblNotFound(self.track_id(), BoxType::Mp4aBox))
        }
    }

    fn stsc_index(&self, sample_id: u32) -> Result<usize> {
        if self.trak.mdia.minf.stbl.stsc.entries.is_empty() {
            return Err(Error::InvalidData("no stsc entries"));
        }
        for (i, entry) in self.trak.mdia.minf.stbl.stsc.entries.iter().enumerate() {
            if sample_id < entry.first_sample {
                return if i == 0 {
                    Err(Error::InvalidData("sample not found"))
                } else {
                    Ok(i - 1)
                };
            }
        }
        Ok(self.trak.mdia.minf.stbl.stsc.entries.len() - 1)
    }

    fn chunk_offset(&self, chunk_id: u32) -> Result<u64> {
        if self.trak.mdia.minf.stbl.stco.is_none() && self.trak.mdia.minf.stbl.co64.is_none() {
            return Err(Error::InvalidData("must have either stco or co64 boxes"));
        }
        if let Some(ref stco) = self.trak.mdia.minf.stbl.stco {
            if let Some(offset) = stco.entries.get(chunk_id as usize - 1) {
                return Ok(*offset as u64);
            } else {
                return Err(Error::EntryInStblNotFound(
                    self.track_id(),
                    BoxType::StcoBox,
                    chunk_id,
                ));
            }
        } else if let Some(ref co64) = self.trak.mdia.minf.stbl.co64 {
            if let Some(offset) = co64.entries.get(chunk_id as usize - 1) {
                return Ok(*offset);
            } else {
                return Err(Error::EntryInStblNotFound(
                    self.track_id(),
                    BoxType::Co64Box,
                    chunk_id,
                ));
            }
        }
        Err(Error::Box2NotFound(BoxType::StcoBox, BoxType::Co64Box))
    }

    fn ctts_index(&self, sample_id: u32) -> Result<(usize, u32)> {
        let ctts = self.trak.mdia.minf.stbl.ctts.as_ref().unwrap();
        let mut sample_count = 1;
        for (i, entry) in ctts.entries.iter().enumerate() {
            if sample_id < sample_count + entry.sample_count {
                return Ok((i, sample_count));
            }
            sample_count += entry.sample_count;
        }

        Err(Error::EntryInStblNotFound(
            self.track_id(),
            BoxType::CttsBox,
            sample_id,
        ))
    }

    /// return `(traf_idx, sample_idx_in_trun)`
    fn find_traf_idx_and_sample_idx(&self, sample_id: u32) -> Option<(usize, usize)> {
        let global_idx = sample_id - 1;
        let mut offset = 0;
        for traf_idx in 0..self.trafs.len() {
            if let Some(trun) = &self.trafs[traf_idx].trun {
                let sample_count = trun.sample_count;
                if sample_count > (global_idx - offset) {
                    return Some((traf_idx, (global_idx - offset) as _));
                }
                offset += sample_count;
            }
        }
        None
    }

    fn sample_size(&self, sample_id: u32) -> Result<u32> {
        if !self.trafs.is_empty() {
            if let Some((traf_idx, sample_idx)) = self.find_traf_idx_and_sample_idx(sample_id) {
                if let Some(size) = self.trafs[traf_idx]
                    .trun
                    .as_ref()
                    .unwrap()
                    .sample_sizes
                    .get(sample_idx)
                {
                    Ok(*size)
                } else {
                    Err(Error::EntryInTrunNotFound(
                        self.track_id(),
                        BoxType::TrunBox,
                        sample_id,
                    ))
                }
            } else {
                Err(Error::BoxInTrafNotFound(self.track_id(), BoxType::TrafBox))
            }
        } else {
            let stsz = &self.trak.mdia.minf.stbl.stsz;
            if stsz.sample_size > 0 {
                return Ok(stsz.sample_size);
            }
            if let Some(size) = stsz.sample_sizes.get(sample_id as usize - 1) {
                Ok(*size)
            } else {
                Err(Error::EntryInStblNotFound(
                    self.track_id(),
                    BoxType::StszBox,
                    sample_id,
                ))
            }
        }
    }

    fn total_sample_size(&self) -> u64 {
        let stsz = &self.trak.mdia.minf.stbl.stsz;
        if stsz.sample_size > 0 {
            stsz.sample_size as u64 * self.sample_count() as u64
        } else {
            let mut total_size = 0;
            for size in stsz.sample_sizes.iter() {
                total_size += *size as u64;
            }
            total_size
        }
    }

    fn sample_offset(&self, sample_id: u32) -> Result<u64> {
        if !self.trafs.is_empty() {
            if let Some((traf_idx, _sample_idx)) = self.find_traf_idx_and_sample_idx(sample_id) {
                Ok(self.trafs[traf_idx].tfhd.base_data_offset as u64)
            } else {
                Err(Error::BoxInTrafNotFound(self.track_id(), BoxType::TrafBox))
            }
        } else {
            let stsc_index = self.stsc_index(sample_id)?;

            let stsc = &self.trak.mdia.minf.stbl.stsc;
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
    }

    fn sample_time(&self, sample_id: u32) -> Result<(u64, u32)> {
        let stts = &self.trak.mdia.minf.stbl.stts;

        let mut sample_count = 1;
        let mut elapsed = 0;

        if !self.trafs.is_empty() {
            let start_time = ((sample_id - 1) * self.default_sample_duration) as u64;
            Ok((start_time, self.default_sample_duration))
        } else {
            for entry in stts.entries.iter() {
                if sample_id < sample_count + entry.sample_count {
                    let start_time =
                        (sample_id - sample_count) as u64 * entry.sample_delta as u64 + elapsed;
                    return Ok((start_time, entry.sample_delta));
                }

                sample_count += entry.sample_count;
                elapsed += entry.sample_count as u64 * entry.sample_delta as u64;
            }

            Err(Error::EntryInStblNotFound(
                self.track_id(),
                BoxType::SttsBox,
                sample_id,
            ))
        }
    }

    fn sample_rendering_offset(&self, sample_id: u32) -> i32 {
        if let Some(ref ctts) = self.trak.mdia.minf.stbl.ctts {
            if let Ok((ctts_index, _)) = self.ctts_index(sample_id) {
                let ctts_entry = ctts.entries.get(ctts_index).unwrap();
                return ctts_entry.sample_offset;
            }
        }
        0
    }

    fn is_sync_sample(&self, sample_id: u32) -> bool {
        if !self.trafs.is_empty() {
            let sample_sizes_count = self.sample_count() / self.trafs.len() as u32;
            return sample_id == 1 || sample_id % sample_sizes_count == 0;
        }

        if let Some(ref stss) = self.trak.mdia.minf.stbl.stss {
            match stss.entries.binary_search(&sample_id) {
                Ok(_) => true,
                Err(_) => false,
            }
        } else {
            true
        }
    }

    pub(crate) fn read_sample<R: Read + Seek>(
        &self,
        reader: &mut R,
        sample_id: u32,
    ) -> Result<Option<Mp4Sample>> {
        let sample_offset = match self.sample_offset(sample_id) {
            Ok(offset) => offset,
            Err(Error::EntryInStblNotFound(_, _, _)) => return Ok(None),
            Err(err) => return Err(err),
        };
        let sample_size = self.sample_size(sample_id).unwrap();

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

// TODO creation_time, modification_time
#[derive(Debug, Default)]
pub(crate) struct Mp4TrackWriter {
    trak: TrakBox,

    sample_id: u32,
    fixed_sample_size: u32,
    is_fixed_sample_size: bool,
    chunk_samples: u32,
    chunk_duration: u32,
    chunk_buffer: BytesMut,

    samples_per_chunk: u32,
    duration_per_chunk: u32,
}

impl Mp4TrackWriter {
    pub(crate) fn new(track_id: u32, config: &TrackConfig) -> Result<Self> {
        let mut trak = TrakBox::default();
        trak.tkhd.track_id = track_id;
        trak.mdia.mdhd.timescale = config.timescale;
        trak.mdia.mdhd.language = config.language.to_owned();
        trak.mdia.hdlr.handler_type = config.track_type.into();
        trak.mdia.minf.stbl.co64 = Some(Co64Box::default());
        match config.media_conf {
            MediaConfig::AvcConfig(ref avc_config) => {
                trak.tkhd.set_width(avc_config.width);
                trak.tkhd.set_height(avc_config.height);

                let vmhd = VmhdBox::default();
                trak.mdia.minf.vmhd = Some(vmhd);

                let avc1 = Avc1Box::new(avc_config);
                trak.mdia.minf.stbl.stsd.avc1 = Some(avc1);
            }
            MediaConfig::HevcConfig(ref hevc_config) => {
                trak.tkhd.set_width(hevc_config.width);
                trak.tkhd.set_height(hevc_config.height);

                let vmhd = VmhdBox::default();
                trak.mdia.minf.vmhd = Some(vmhd);

                let hev1 = Hev1Box::new(hevc_config);
                trak.mdia.minf.stbl.stsd.hev1 = Some(hev1);
            }
            MediaConfig::Vp9Config(ref config) => {
                trak.tkhd.set_width(config.width);
                trak.tkhd.set_height(config.height);

                trak.mdia.minf.stbl.stsd.vp09 = Some(Vp09Box::new(config));
            }
            MediaConfig::AacConfig(ref aac_config) => {
                let smhd = SmhdBox::default();
                trak.mdia.minf.smhd = Some(smhd);

                let mp4a = Mp4aBox::new(aac_config);
                trak.mdia.minf.stbl.stsd.mp4a = Some(mp4a);
            }
            MediaConfig::TtxtConfig(ref _ttxt_config) => {
                let tx3g = Tx3gBox::default();
                trak.mdia.minf.stbl.stsd.tx3g = Some(tx3g);
            }
        }
        Ok(Mp4TrackWriter {
            trak,
            chunk_buffer: BytesMut::new(),
            sample_id: 1,
            duration_per_chunk: config.timescale, // 1 second
            ..Self::default()
        })
    }

    fn update_sample_sizes(&mut self, size: u32) {
        if self.trak.mdia.minf.stbl.stsz.sample_count == 0 {
            if size == 0 {
                self.trak.mdia.minf.stbl.stsz.sample_size = 0;
                self.is_fixed_sample_size = false;
                self.trak.mdia.minf.stbl.stsz.sample_sizes.push(0);
            } else {
                self.trak.mdia.minf.stbl.stsz.sample_size = size;
                self.fixed_sample_size = size;
                self.is_fixed_sample_size = true;
            }
        } else if self.is_fixed_sample_size {
            if self.fixed_sample_size != size {
                self.is_fixed_sample_size = false;
                if self.trak.mdia.minf.stbl.stsz.sample_size > 0 {
                    self.trak.mdia.minf.stbl.stsz.sample_size = 0;
                    for _ in 0..self.trak.mdia.minf.stbl.stsz.sample_count {
                        self.trak
                            .mdia
                            .minf
                            .stbl
                            .stsz
                            .sample_sizes
                            .push(self.fixed_sample_size);
                    }
                }
                self.trak.mdia.minf.stbl.stsz.sample_sizes.push(size);
            }
        } else {
            self.trak.mdia.minf.stbl.stsz.sample_sizes.push(size);
        }
        self.trak.mdia.minf.stbl.stsz.sample_count += 1;
    }

    fn update_sample_times(&mut self, dur: u32) {
        if let Some(ref mut entry) = self.trak.mdia.minf.stbl.stts.entries.last_mut() {
            if entry.sample_delta == dur {
                entry.sample_count += 1;
                return;
            }
        }

        let entry = SttsEntry {
            sample_count: 1,
            sample_delta: dur,
        };
        self.trak.mdia.minf.stbl.stts.entries.push(entry);
    }

    fn update_rendering_offsets(&mut self, offset: i32) {
        let ctts = if let Some(ref mut ctts) = self.trak.mdia.minf.stbl.ctts {
            ctts
        } else {
            if offset == 0 {
                return;
            }
            let mut ctts = CttsBox::default();
            if self.sample_id > 1 {
                let entry = CttsEntry {
                    sample_count: self.sample_id - 1,
                    sample_offset: 0,
                };
                ctts.entries.push(entry);
            }
            self.trak.mdia.minf.stbl.ctts = Some(ctts);
            self.trak.mdia.minf.stbl.ctts.as_mut().unwrap()
        };

        if let Some(ref mut entry) = ctts.entries.last_mut() {
            if entry.sample_offset == offset {
                entry.sample_count += 1;
                return;
            }
        }

        let entry = CttsEntry {
            sample_count: 1,
            sample_offset: offset,
        };
        ctts.entries.push(entry);
    }

    fn update_sync_samples(&mut self, is_sync: bool) {
        if let Some(ref mut stss) = self.trak.mdia.minf.stbl.stss {
            if !is_sync {
                return;
            }

            stss.entries.push(self.sample_id);
        } else {
            if !is_sync {
                return;
            }

            // Create the stts box if not found and push the entry.
            let mut stss = StssBox::default();
            stss.entries.push(self.sample_id);
            self.trak.mdia.minf.stbl.stss = Some(stss);
        };
    }

    fn is_chunk_full(&self) -> bool {
        if self.samples_per_chunk > 0 {
            self.chunk_samples >= self.samples_per_chunk
        } else {
            self.chunk_duration >= self.duration_per_chunk
        }
    }

    fn update_durations(&mut self, dur: u32, movie_timescale: u32) {
        self.trak.mdia.mdhd.duration += dur as u64;
        self.trak.tkhd.duration +=
            dur as u64 * movie_timescale as u64 / self.trak.mdia.mdhd.timescale as u64;
    }

    pub(crate) fn write_sample<W: Write + Seek>(
        &mut self,
        writer: &mut W,
        sample: &Mp4Sample,
        movie_timescale: u32,
    ) -> Result<u64> {
        self.chunk_buffer.extend_from_slice(&sample.bytes);
        self.chunk_samples += 1;
        self.chunk_duration += sample.duration;
        self.update_sample_sizes(sample.bytes.len() as u32);
        self.update_sample_times(sample.duration);
        self.update_rendering_offsets(sample.rendering_offset);
        self.update_sync_samples(sample.is_sync);
        if self.is_chunk_full() {
            self.write_chunk(writer)?;
        }
        self.update_durations(sample.duration, movie_timescale);

        self.sample_id += 1;

        Ok(self.trak.tkhd.duration)
    }

    fn chunk_count(&self) -> u32 {
        let co64 = self.trak.mdia.minf.stbl.co64.as_ref().unwrap();
        co64.entries.len() as u32
    }

    fn update_sample_to_chunk(&mut self, chunk_id: u32) {
        if let Some(entry) = self.trak.mdia.minf.stbl.stsc.entries.last() {
            if entry.samples_per_chunk == self.chunk_samples {
                return;
            }
        }

        let entry = StscEntry {
            first_chunk: chunk_id,
            samples_per_chunk: self.chunk_samples,
            sample_description_index: 1,
            first_sample: self.sample_id - self.chunk_samples + 1,
        };
        self.trak.mdia.minf.stbl.stsc.entries.push(entry);
    }

    fn update_chunk_offsets(&mut self, offset: u64) {
        let co64 = self.trak.mdia.minf.stbl.co64.as_mut().unwrap();
        co64.entries.push(offset);
    }

    fn write_chunk<W: Write + Seek>(&mut self, writer: &mut W) -> Result<()> {
        if self.chunk_buffer.is_empty() {
            return Ok(());
        }
        let chunk_offset = writer.seek(SeekFrom::Current(0))?;

        writer.write_all(&self.chunk_buffer)?;

        self.update_sample_to_chunk(self.chunk_count() + 1);
        self.update_chunk_offsets(chunk_offset);

        self.chunk_buffer.clear();
        self.chunk_samples = 0;
        self.chunk_duration = 0;

        Ok(())
    }

    fn max_sample_size(&self) -> u32 {
        if self.trak.mdia.minf.stbl.stsz.sample_size > 0 {
            self.trak.mdia.minf.stbl.stsz.sample_size
        } else {
            let mut max_size = 0;
            for sample_size in self.trak.mdia.minf.stbl.stsz.sample_sizes.iter() {
                max_size = cmp::max(max_size, *sample_size);
            }
            max_size
        }
    }

    pub(crate) fn write_end<W: Write + Seek>(&mut self, writer: &mut W) -> Result<TrakBox> {
        self.write_chunk(writer)?;

        let max_sample_size = self.max_sample_size();
        if let Some(ref mut mp4a) = self.trak.mdia.minf.stbl.stsd.mp4a {
            if let Some(ref mut esds) = mp4a.esds {
                esds.es_desc.dec_config.buffer_size_db = max_sample_size;
            }
            // TODO
            // mp4a.esds.es_desc.dec_config.max_bitrate
            // mp4a.esds.es_desc.dec_config.avg_bitrate
        }
        if let Ok(stco) = StcoBox::try_from(self.trak.mdia.minf.stbl.co64.as_ref().unwrap()) {
            self.trak.mdia.minf.stbl.stco = Some(stco);
            self.trak.mdia.minf.stbl.co64 = None;
        }

        Ok(self.trak.clone())
    }
}
