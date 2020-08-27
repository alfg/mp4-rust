# mp4rs
> MP4 Reader in Rust

ISO/IEC 14496-12 - ISO Base Media File Format (QuickTime, MPEG-4, etc)

[![Crates.io](https://img.shields.io/crates/v/mp4)](https://crates.io/crates/mp4)
[![Crates.io](https://img.shields.io/crates/d/mp4)](https://crates.io/crates/mp4)
[![Build Status](https://travis-ci.org/alfg/mp4rs.svg?branch=master)](https://travis-ci.org/alfg/mp4rs)
![Rust](https://github.com/alfg/mp4rs/workflows/Rust/badge.svg)

#### Example
```rust
use std::fs::File;
use std::io::{BufReader};
use mp4::{Result};

fn main() -> Result<()> {
    let f = File::open("example.mp4").unwrap();
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


## Resources
Thanks to the following resources used when learning Rust:
* https://github.com/mozilla/mp4parse-rust
* https://github.com/pcwalton/rust-media
* https://github.com/alfg/mp4

## License
MIT
