use byteorder::{BigEndian, WriteBytesExt};
use std::io::{Seek, SeekFrom, Write};

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

impl<W> Mp4Writer<W> {
    /// Consume self, returning the inner writer.
    ///
    /// This can be useful to recover the inner writer after completion in case
    /// it's owned by the [Mp4Writer] instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mp4::{Mp4Writer, Mp4Config};
    /// use std::io::Cursor;
    ///
    /// # fn main() -> mp4::Result<()> {
    /// let config = Mp4Config {
    ///     major_brand: str::parse("isom").unwrap(),
    ///     minor_version: 512,
    ///     compatible_brands: vec![
    ///         str::parse("isom").unwrap(),
    ///         str::parse("iso2").unwrap(),
    ///         str::parse("avc1").unwrap(),
    ///         str::parse("mp41").unwrap(),
    ///     ],
    ///     timescale: 1000,
    /// };
    ///
    /// let data = Cursor::new(Vec::<u8>::new());
    /// let mut writer = mp4::Mp4Writer::write_start(data, &config)?;
    /// writer.write_end()?;
    ///
    /// let data: Vec<u8> = writer.into_writer().into_inner();
    /// # Ok(()) }
    /// ```
    pub fn into_writer(self) -> W {
        self.writer
    }
}

impl<W: Write + Seek> Mp4Writer<W> {
    pub fn write_start(mut writer: W, config: &Mp4Config) -> Result<Self> {
        let ftyp = FtypBox {
            major_brand: config.major_brand,
            minor_version: config.minor_version,
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
        if mdat_size > std::u32::MAX as u64 {
            return Err(Error::InvalidData("mdat size too large"));
        }
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
        if moov.mvhd.duration > (u32::MAX as u64) {
            moov.mvhd.version = 1
        }
        moov.write_box(&mut self.writer)?;
        Ok(())
    }
}
