use std::io::{Write, Seek, SeekFrom};

use crate::atoms::*;
use crate::*;

#[derive(Debug)]
pub struct Mp4Writer<W> {
    writer: W,
    tracks: Vec<Mp4Track>,
    mdat_pos: u64,
}

impl<W: Write + Seek> Mp4Writer<W> {
    pub fn write_header(mut writer: W, major_brand: &FourCC, minor_version: u32, compatible_brands: &[FourCC]) -> Result<Self> {
        let ftyp = FtypBox {
            major_brand: major_brand.to_owned(),
            minor_version,
            compatible_brands: compatible_brands.to_vec(),
        };
        ftyp.write_box(&mut writer)?;

        // TODO largesize
        let mdat_pos = writer.seek(SeekFrom::Current(0))?;
        BoxHeader::new(BoxType::MdatBox, HEADER_SIZE).write(&mut writer)?;

        let tracks = Vec::new();
        Ok(Self {
            writer,
            tracks,
            mdat_pos,
        })
    }

    pub fn add_track(&mut self, config: &TrackConfig) -> Result<()> {
        let track_id = self.tracks.len() as u32 + 1;
        let track = Mp4Track::new(track_id, config)?;
        self.tracks.push(track);
        Ok(())
    }

    pub fn write_tail(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn write_sample(&mut self, _track_id: u32, _sample: &Mp4Sample) -> Result<()> {
        Ok(())
    }
}
