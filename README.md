# mp4rs
> MP4 Reader in Rust

ISO/IEC 14496-12 - ISO Base Media File Format (QuickTime, MPEG-4, etc)

[![Crates.io](https://img.shields.io/crates/v/mp4)](https://crates.io/crates/mp4)
[![Crates.io](https://img.shields.io/crates/d/mp4)](https://crates.io/crates/mp4)
[![Build Status](https://travis-ci.org/alfg/mp4rs.svg?branch=master)](https://travis-ci.org/alfg/mp4rs)
![Rust](https://github.com/alfg/mp4rs/workflows/Rust/badge.svg)

#### Example
```rust
use mp4;

fn main() {
    let f = File::open("example.mp4").unwrap();
    let size = f.metadata()?.len();
    let reader = BufReader::new(f);

    let mut mp4 = Mp4Reader::new(reader);
    mp4.read(size)?;

    println!("size: {}", mp4.size());
    println!("brands: {:?} {:?}\n", mp4.ftyp.major_brand, mp4.ftyp.compatible_brands);
}
```

See [examples/](examples/) for a full example.

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

## Resources
Thanks to the following resources used when learning Rust:
* https://github.com/mozilla/mp4parse-rust
* https://github.com/pcwalton/rust-media
* https://github.com/alfg/mp4

## License
MIT
