use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fast_blurhash::base83;

static CHARACTERS: [u8; 83] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F',
    b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V',
    b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l',
    b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z', b'#', b'$',
    b'%', b'*', b'+', b',', b'-', b'.', b':', b';', b'=', b'?', b'@', b'[', b']', b'^', b'_', b'{',
    b'|', b'}', b'~',
];

pub fn encode(value: u32, length: u32) -> String {
    let mut result = String::new();

    for i in 1..=length {
        let digit: u32 = (value / u32::pow(83, length - i)) % 83;
        result.push(CHARACTERS[digit as usize] as char);
    }

    result
}

pub fn decode(str: &str) -> Result<usize, u8> {
    let mut value = 0;

    for byte in str.as_bytes() {
        let digit: usize = CHARACTERS
            .iter()
            .position(|r| r == byte)
            .ok_or_else(|| (*byte))?;
        value = value * 83 + digit;
    }

    Ok(value)
}

fn encode_benches(c: &mut Criterion) {
    let mut s = String::with_capacity(4);
    c.bench_function("my encode max [no alloc]", |b| b.iter(|| {
        base83::encode_to(black_box(u32::MAX), black_box(&mut s));
        s.clear();
    }));
    c.bench_function("my encode max [alloc]", |b| b.iter(|| {
        base83::encode(black_box(u32::MAX));
    }));
    c.bench_function("blurhash:1.1.0 encode [alloc]", |b| b.iter(|| {
        encode(black_box(u32::MAX), black_box(6));
    }));
}

fn decode_benches(c: &mut Criterion) {
    c.bench_function("my decode", |b| b.iter(|| {
        base83::decode(black_box("17fd^]"));
    }));
    c.bench_function("blurhash:1.1.0 decode", |b| b.iter(|| {
        decode(black_box("17fd^]"));
    }));
}

criterion_group!(benches, encode_benches, decode_benches);
criterion_main!(benches);
