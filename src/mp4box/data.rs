use std::{
    convert::TryFrom,
    io::{Read, Seek},
};

use serde::Serialize;

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct DataBox {
    pub data: Vec<u8>,
    pub data_type: DataType,
}

impl<R: Read + Seek> ReadBox<&mut R> for DataBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let data_type = DataType::try_from(reader.read_u32::<BigEndian>()?)?;

        reader.read_u32::<BigEndian>()?; // reserved = 0

        let current = reader.seek(SeekFrom::Current(0))?;
        let mut data = vec![0u8; (start + size - current) as usize];
        reader.read_exact(&mut data)?;

        Ok(DataBox { data, data_type })
    }
}
