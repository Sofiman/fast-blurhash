use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fast_blurhash::base83;

fn encode_benches(c: &mut Criterion) {
    c.bench_function("encode [alloc]", |b| b.iter(|| {
        base83::encode(black_box(u32::MAX));
    }));
    let mut s = String::with_capacity(6);
    c.bench_function("encode_to [no alloc]", |b| b.iter(|| {
        base83::encode_to(black_box(u32::MAX), black_box(&mut s));
        s.clear();
    }));
    c.bench_function("encode_fixed_to [no alloc]", |b| b.iter(|| {
        base83::encode_fixed_to(black_box(u32::MAX), 6, black_box(&mut s));
        s.clear();
    }));
}

fn decode_benches(c: &mut Criterion) {
    c.bench_function("decode_ascii", |b| b.iter(|| {
        base83::decode_ascii(black_box("17fd^]"));
    }));
    c.bench_function("my decode", |b| b.iter(|| {
        base83::decode(black_box("17fd^]"));
    }));
}

criterion_group!(benches, encode_benches, decode_benches);
criterion_main!(benches);
