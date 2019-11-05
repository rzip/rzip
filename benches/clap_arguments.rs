#[macro_use]
extern crate clap;

use criterion::{criterion_group, criterion_main, Criterion};

fn load_yaml() {
    load_yaml!("../cli_def/en_us.yml");
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("loading en_us.yml lang definition", |b| b.iter(load_yaml));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
