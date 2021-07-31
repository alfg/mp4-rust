# mp4
> MP4 Reader and Writer in Rust 🦀

`mp4` is a Rust library to read and write ISO-MP4 files. This package contains MPEG-4 specifications defined in parts:
* [ISO/IEC 14496-12](https://en.wikipedia.org/wiki/ISO/IEC_base_media_file_format) - ISO Base Media File Format (QuickTime, MPEG-4, etc)
* [ISO/IEC 14496-14](https://en.wikipedia.org/wiki/MPEG-4_Part_14) - MP4 file format
* ISO/IEC 14496-17 - Streaming text format

https://crates.io/crates/mp4

[![Crates.io](https://img.shields.io/crates/v/mp4)](https://crates.io/crates/mp4)
[![Crates.io](https://img.shields.io/crates/d/mp4)](https://crates.io/crates/mp4)
[![Docs](https://img.shields.io/badge/docs-online-5023dd.svg?style=flat-square)](https://docs.rs/mp4)
[![Build Status](https://travis-ci.org/alfg/mp4-rust.svg?branch=master)](https://travis-ci.org/alfg/mp4-rust)
[![Rust](https://github.com/alfg/mp4-rust/workflows/Rust/badge.svg)](https://github.com/alfg/mp4-rust/actions)

#### Example
```rust
use std::fs::File;
use std::io::{BufReader};
use mp4::{Result};

fn main() -> Result<()> {
    let f = File::open("tests/samples/minimal.mp4").unwrap();
    let size = f.metadata()?.len();
    let reader = BufReader::new(f);

    let mp4 = mp4::Mp4Reader::read_header(reader, size)?;

    // Print boxes.
    println!("major brand: {}", mp4.ftyp.major_brand);
    println!("timescale: {}", mp4.moov.mvhd.timescale);

    // Use available methods.
    println!("size: {}", mp4.size());

    let mut compatible_brands = String::new();
    for brand in mp4.compatible_brands().iter() {
        compatible_brands.push_str(&brand.to_string());
        compatible_brands.push_str(",");
    }
    println!("compatible brands: {}", compatible_brands);
    println!("duration: {:?}", mp4.duration());

    // Track info.
    for track in mp4.tracks().iter() {
        println!(
            "track: #{}({}) {} : {}",
            track.track_id(),
            track.language(),
            track.track_type()?,
            track.box_type()?,
        );
    }
    Ok(())
}
```

See [examples/](examples/) for more examples.

#### Install
Add to your `Cargo.toml`:
```toml
mp4 = "0.9.0"
```

#### Documentation
* https://docs.rs/mp4/

## Development

#### Requirements
* [Rust](https://www.rust-lang.org/)

#### Build
```
cargo build
```

#### Run Examples
* `mp4info`
```
cargo run --example mp4info <movie.mp4>
```

* `mp4dump`
```
cargo run --example mp4dump <movie.mp4>
```

#### Run Tests
```
cargo test
```

With print statement output.
```
cargo test -- --nocapture
```

#### Run Benchmark Tests
```
cargo bench
```

View HTML report at `target/criterion/report/index.html`

#### Generate Docs
```
cargo docs
```

View at `target/doc/mp4/index.html`

## Web Assembly
See the [mp4-inspector](https://github.com/alfg/mp4-inspector) project as a reference for using this library in Javascript via Web Assembly.

## Related Projects
* https://github.com/mozilla/mp4parse-rust
* https://github.com/pcwalton/rust-media
* https://github.com/alfg/mp4

## License
MIT

[docs]: https://docs.rs/mp4
[docs-badge]: https://img.shields.io/badge/docs-online-5023dd.svg?style=flat-square
