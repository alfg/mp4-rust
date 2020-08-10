use byteorder::{BigEndian, WriteBytesExt};
use std::io::{Seek, SeekFrom, Write};

#[cfg(feature = "async")]
use {
    std::marker::Unpin,
    std::io::Cursor,
    tokio::io::{AsyncSeekExt, AsyncWriteExt},
};

use crate::mp4box::*;
use crate::track::Mp4TrackWriter;
use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Mp4Config {
    pub major_brand: FourCC,
    pub minor_version: u32,
    pub compatible_brands: Vec<FourCC>,
    pub timescale: u32,
}

#[derive(Debug)]
pub struct Mp4Writer<W> {
    writer: W,
    tracks: Vec<Mp4TrackWriter>,
    mdat_pos: u64,
    timescale: u32,
    duration: u64,
}

impl<W: Write + Seek> Mp4Writer<W> {
    pub fn write_start(mut writer: W, config: &Mp4Config) -> Result<Self> {
        let ftyp = FtypBox {
            major_brand: config.major_brand.clone(),
            minor_version: config.minor_version.clone(),
            compatible_brands: config.compatible_brands.clone(),
        };
        ftyp.write_box(&mut writer)?;

        // TODO largesize
        let mdat_pos = writer.seek(SeekFrom::Current(0))?;
        BoxHeader::new(BoxType::MdatBox, HEADER_SIZE).write(&mut writer)?;

        let tracks = Vec::new();
        let timescale = config.timescale;
        let duration = 0;
        Ok(Self {
            writer,
            tracks,
            mdat_pos,
            timescale,
            duration,
        })
    }

    pub fn add_track(&mut self, config: &TrackConfig) -> Result<()> {
        let track_id = self.tracks.len() as u32 + 1;
        let track = Mp4TrackWriter::new(track_id, config)?;
        self.tracks.push(track);
        Ok(())
    }

    fn update_durations(&mut self, track_dur: u64) {
        if track_dur > self.duration {
            self.duration = track_dur;
        }
    }

    pub fn write_sample(&mut self, track_id: u32, sample: &Mp4Sample) -> Result<()> {
        if track_id == 0 {
            return Err(Error::TrakNotFound(track_id));
        }

        let track_dur = if let Some(ref mut track) = self.tracks.get_mut(track_id as usize - 1) {
            track.write_sample(&mut self.writer, sample, self.timescale)?
        } else {
            return Err(Error::TrakNotFound(track_id));
        };

        self.update_durations(track_dur);

        Ok(())
    }

    fn update_mdat_size(&mut self) -> Result<()> {
        let mdat_end = self.writer.seek(SeekFrom::Current(0))?;
        let mdat_size = mdat_end - self.mdat_pos;
        assert!(mdat_size < std::u32::MAX as u64);
        self.writer.seek(SeekFrom::Start(self.mdat_pos))?;
        self.writer.write_u32::<BigEndian>(mdat_size as u32)?;
        self.writer.seek(SeekFrom::Start(mdat_end))?;
        Ok(())
    }

    pub fn write_end(&mut self) -> Result<()> {
        let mut moov = MoovBox::default();

        for track in self.tracks.iter_mut() {
            moov.traks.push(track.write_end(&mut self.writer)?);
        }
        self.update_mdat_size()?;

        moov.mvhd.timescale = self.timescale;
        moov.mvhd.duration = self.duration;
        moov.write_box(&mut self.writer)?;
        Ok(())
    }
}

#[cfg(feature = "async")]
#[derive(Debug)]
pub struct Mp4AsyncWriter<W> {
    writer: W,
    tracks: Vec<Mp4TrackWriter>,
    mdat_pos: u64,
    timescale: u32,
    duration: u64,
}

#[cfg(feature = "async")]
impl<W> Mp4AsyncWriter<W>
where
    W: AsyncWriteExt + AsyncSeekExt + Unpin
{
    pub async fn async_write_start(mut writer: W, config: &Mp4Config) -> Result<Self> {
        let ftyp = FtypBox {
            major_brand: config.major_brand.clone(),
            minor_version: config.minor_version.clone(),
            compatible_brands: config.compatible_brands.clone(),
        };
        let mut buffer = vec![0u8; ftyp.box_size() as usize];
        ftyp.write_box(&mut Cursor::new(&mut buffer))?;
        writer.write_all(&buffer).await?;

        // TODO largesize
        let mdat_pos = writer.seek(SeekFrom::Current(0)).await?;
        BoxHeader::new(BoxType::MdatBox, HEADER_SIZE).async_write(&mut writer).await?;

        let tracks = Vec::new();
        let timescale = config.timescale;
        let duration = 0;
        Ok(Self {
            writer,
            tracks,
            mdat_pos,
            timescale,
            duration,
        })
    }

    pub fn add_track(&mut self, config: &TrackConfig) -> Result<()> {
        let track_id = self.tracks.len() as u32 + 1;
        let track = Mp4TrackWriter::new(track_id, config)?;
        self.tracks.push(track);
        Ok(())
    }

    fn update_durations(&mut self, track_dur: u64) {
        if track_dur > self.duration {
            self.duration = track_dur;
        }
    }

    pub async fn async_write_sample(&mut self, track_id: u32, sample: &Mp4Sample) -> Result<()> {
        if track_id == 0 {
            return Err(Error::TrakNotFound(track_id));
        }

        let track_dur = if let Some(ref mut track) = self.tracks.get_mut(track_id as usize - 1) {
            track.async_write_sample(&mut self.writer, sample, self.timescale).await?
        } else {
            return Err(Error::TrakNotFound(track_id));
        };

        self.update_durations(track_dur);

        Ok(())
    }

    async fn async_update_mdat_size(&mut self) -> Result<()> {
        let mdat_end = self.writer.seek(SeekFrom::Current(0)).await?;
        let mdat_size = mdat_end - self.mdat_pos;
        assert!(mdat_size < std::u32::MAX as u64);
        self.writer.seek(SeekFrom::Start(self.mdat_pos)).await?;
        self.writer.write_all(&(mdat_size as u32).to_be_bytes()).await?;
        self.writer.seek(SeekFrom::Start(mdat_end)).await?;
        Ok(())
    }

    pub async fn async_write_end(&mut self) -> Result<()> {
        let mut moov = MoovBox::default();

        for track in self.tracks.iter_mut() {
            moov.traks.push(track.async_write_end(&mut self.writer).await?);
        }
        self.async_update_mdat_size().await?;

        moov.mvhd.timescale = self.timescale;
        moov.mvhd.duration = self.duration;

        let mut buffer = vec![0u8; moov.box_size() as usize];
        moov.write_box(&mut Cursor::new(&mut buffer))?;
        self.writer.write_all(&buffer).await?;

        Ok(())
    }
}
