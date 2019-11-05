use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn parse_zip_10_mb(path: &str) -> zip::read::ZipArchive<std::fs::File> {
    let fname = std::path::Path::new(path);
    let file = std::fs::File::open(&fname).unwrap();
    zip::ZipArchive::new(file).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parsing zip_10MB", |b| {
        b.iter(|| parse_zip_10_mb(black_box("tests/zip_10MB.zip")))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
