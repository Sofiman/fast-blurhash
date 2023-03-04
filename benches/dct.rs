use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fast_blurhash::{compute_dct, compute_dct_iter, encode, decode};
use ril::prelude::Image;

fn blurhash_benches(c: &mut Criterion) {
    let img = Image::<ril::pixel::Rgb>::open("test.webp").unwrap();
    let w = img.width() as usize;
    let h = img.height() as usize;

    let pixels: Vec<fast_blurhash::Rgb> = img.pixels().flatten()
        .map(|p| [p.r, p.g, p.b]).collect();

    let dct = compute_dct(&pixels, w, h, 4, 7);

    c.bench_function("compute_dct_iter", |b| b.iter(|| {
        compute_dct_iter(black_box(pixels.iter()), black_box(w), black_box(h), black_box(4), black_box(7));
    }));

    c.bench_function("compute_dct", |b| b.iter(|| {
        compute_dct(black_box(&pixels), black_box(w), black_box(h), black_box(4), black_box(7));
    }));

    c.bench_function("encode_blurhash", |b| b.iter(|| {
        encode(black_box(&dct));
    }));

    let blurhash = encode(&dct);
    c.bench_function("decode_blurhash", |b| b.iter(|| {
        decode(black_box(&blurhash), 1.);
    }));

    c.bench_function("to_image", |b| b.iter(|| {
        black_box(&dct).to_image(32, 48, |f| f);
    }));

    c.bench_function("to_rgba", |b| b.iter(|| {
        black_box(&dct).to_rgba(32, 48);
    }));
}

criterion_group!(benches, blurhash_benches);
criterion_main!(benches);
