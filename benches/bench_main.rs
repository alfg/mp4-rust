use criterion::BenchmarkId;
use criterion::{criterion_group, criterion_main, Criterion};

use std::fs::File;

fn read_mp4(filename: &str) -> u64 {
    let f = File::open(filename).unwrap();
    let m = mp4::read_mp4(f).unwrap();

    m.size()
}

fn criterion_benchmark(c: &mut Criterion) {
    let filename = "tests/samples/minimal.mp4";

    c.bench_with_input(
        BenchmarkId::new("input_example", filename),
        &filename,
        |b, &s| {
            b.iter(|| read_mp4(s));
        },
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
