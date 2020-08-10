use std::io::{Read, Seek, SeekFrom};
use std::time::Duration;

#[cfg(feature = "async")]
use {
    std::marker::Unpin,
    std::io::Cursor,
    tokio::io::{AsyncReadExt, AsyncSeekExt},
};

use crate::mp4box::*;
use crate::*;

#[derive(Debug)]
pub struct Mp4Reader<R> {
    reader: R,
    ftyp: FtypBox,
    moov: MoovBox,

    tracks: Vec<Mp4Track>,
    size: u64,
}

impl<R: Read + Seek> Mp4Reader<R> {
    pub fn read_header(mut reader: R, size: u64) -> Result<Self> {
        let start = reader.seek(SeekFrom::Current(0))?;

        let mut ftyp = None;
        let mut moov = None;

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
                    skip_box(&mut reader, s)?;
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
        let tracks = if let Some(ref moov) = moov {
            let mut tracks = Vec::with_capacity(moov.traks.len());
            for (i, trak) in moov.traks.iter().enumerate() {
                assert_eq!(trak.tkhd.track_id, i as u32 + 1);
                tracks.push(Mp4Track::from(trak));
            }
            tracks
        } else {
            Vec::new()
        };

        Ok(Mp4Reader {
            reader,
            ftyp: ftyp.unwrap(),
            moov: moov.unwrap(),
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

#[cfg(feature = "async")]
#[derive(Debug)]
pub struct Mp4AsyncReader<R> {
    reader: R,
    ftyp: FtypBox,
    moov: MoovBox,

    tracks: Vec<Mp4Track>,
    size: u64,
}

#[cfg(feature = "async")]
impl<R> Mp4AsyncReader<R>
where
    R: AsyncReadExt + AsyncSeekExt + Unpin
{
    pub async fn async_read_header(mut reader: R, size: u64) -> Result<Self> {
        let start = reader.seek(SeekFrom::Current(0)).await?;

        let mut ftyp = None;
        let mut moov = None;

        let mut current = start;
        while current < size {
            // Get box header.
            let header = BoxHeader::async_read(&mut reader).await?;
            let BoxHeader { name, size: s } = header;

            // Match and parse the atom boxes.
            match name {
                BoxType::FtypBox => {
                    let mut buffer = vec![0u8; s as usize];
                    reader.read_exact(&mut buffer[HEADER_SIZE as usize..]).await?;
                    let mut cursor = Cursor::new(&mut buffer);
                    skip_bytes(&mut cursor, HEADER_SIZE)?;
                    ftyp = Some(FtypBox::read_box(&mut cursor, s)?);
                }
                BoxType::FreeBox => {
                    async_skip_box(&mut reader, s).await?;
                }
                BoxType::MdatBox => {
                    async_skip_box(&mut reader, s).await?;
                }
                BoxType::MoovBox => {
                    let mut buffer = vec![0u8; s as usize];
                    reader.read_exact(&mut buffer[HEADER_SIZE as usize..]).await?;
                    let mut cursor = Cursor::new(&mut buffer);
                    skip_bytes(&mut cursor, HEADER_SIZE)?;
                    moov = Some(MoovBox::read_box(&mut cursor, s)?);
                }
                BoxType::MoofBox => {
                    async_skip_box(&mut reader, s).await?;
                }
                _ => {
                    // XXX warn!()
                    async_skip_box(&mut reader, s).await?;
                }
            }
            current = reader.seek(SeekFrom::Current(0)).await?;
        }

        if ftyp.is_none() {
            return Err(Error::BoxNotFound(BoxType::FtypBox));
        }
        if moov.is_none() {
            return Err(Error::BoxNotFound(BoxType::MoovBox));
        }

        let size = current - start;
        let tracks = if let Some(ref moov) = moov {
            let mut tracks = Vec::with_capacity(moov.traks.len());
            for (i, trak) in moov.traks.iter().enumerate() {
                assert_eq!(trak.tkhd.track_id, i as u32 + 1);
                tracks.push(Mp4Track::from(trak));
            }
            tracks
        } else {
            Vec::new()
        };

        Ok(Mp4AsyncReader {
            reader,
            ftyp: ftyp.unwrap(),
            moov: moov.unwrap(),
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

    pub async fn async_read_sample(&mut self, track_id: u32, sample_id: u32) -> Result<Option<Mp4Sample>> {
        if track_id == 0 {
            return Err(Error::TrakNotFound(track_id));
        }

        if let Some(ref track) = self.tracks.get(track_id as usize - 1) {
            track.async_read_sample(&mut self.reader, sample_id).await
        } else {
            Err(Error::TrakNotFound(track_id))
        }
    }
}
