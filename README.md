# Fast(er) BlurHash

A faster implementation of the BlurHash algorithm used to generate better
looking placeholder for websites and mobile apps. This crates encode and decode
functions minimizes the number of allocated vectors to reduce the memory footprint.
The base83 encode and decode are also both very fast!

## Usage

Generating a blurhash from an image:
```rust
use fast_blurhash::compute_dct;

let (width, height) = todo!("Get image width and height");
let image: Vec<u32> = todo!("Load the image");
let blurhash = compute_dct(&image, width, height, 3, 4).into_blurhash();
```

You can also pass a long-enough iterator to avoid copying data:
```rust
use fast_blurhash::compute_dct;

let (width, height) = todo!("Get image width and height");
let image: Vec<u32> = todo!("Load the image");
let blurhash = compute_dct_iter(image.iter(), width, height, 3, 4).into_blurhash();
```

Supported types to be used with compute_dct:
| Type | Alias | Disposition | Notes |
|---|---|---|---|
| [f32; 3] | Linear | [Red, Green, Blue] | Channels are in linear space |
| [u8; 3], &[u8; 3] | Rgb | [Red, Green, Blue] |  |
| [u8; 4], &[u8; 4] | Rgba | [Red, Green, Blue, Alpha] | Alpha is ignored |
| u32 |  | 0xAARRGGBB where A is alpha | Alpha is ignored |

> This crate also supports using your custom types (see the trait AsLinear and
> examples in the documentation).

Generating an image from a blurhash:
```rust
use fast_blurhash::decode;

let blurhash = "LlMF%n00%#MwS|WCWEM{R*bbWBbH";
let image: Vec<u32> = decode(&blurhash, 1.).unwrap().to_rgba(32, 32);
```

Available generation functions:
| Function | Return type | Disposition | Notes |
|---|---|---|---|
| to_image<T>(width, height, fn(Linear) -> T) | Vec<T> | Linear: [Red, Green, Blue] | Linear is a builtin type that represents a color in linear space. |
| to_rgb8(width, height) | Vec<[u8; 3]> | [Red, Green, Blue] |  |
| to_rgba8(width, height) | Vec<[u8; u4]> | [Red, Green, Blue, Alpha] | Alpha will always be 255 |
| to_rgba(width, height) | Vec<u32> | 0xAARRGGBB where A is alpha | Alpha will always be 255 |

## Documentation

More documentation is available in rust docs.

### TODO

- [x] Add documentation
- [x] Add decode
- [ ] Publish to crate.io

## Contribution & Feedback

If you have any feedback, please open an issue. If you encounter any bugs or unwanted behaviour, please open an issue.

This projet is open to contributions, feel free to submit your pull requests!

# License
