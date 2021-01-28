use mp4::{Mp4Config, Mp4Writer};
use std::io::Cursor;

fn main() -> mp4::Result<()> {
    let config = Mp4Config {
        major_brand: str::parse("isom").unwrap(),
        minor_version: 512,
        compatible_brands: vec![
            str::parse("isom").unwrap(),
            str::parse("iso2").unwrap(),
            str::parse("avc1").unwrap(),
            str::parse("mp41").unwrap(),
        ],
        timescale: 1000,
    };

    let data = Cursor::new(Vec::<u8>::new());
    let mut writer = Mp4Writer::write_start(data, &config)?;
    writer.write_end()?;

    let data: Vec<u8> = writer.into_writer().into_inner();
    println!("{:?}", data);
    Ok(())
}