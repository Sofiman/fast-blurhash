//! # Fast(er) BlurHash
//!
//! Provides a faster implementation of the BlurHash algorithm for encoding
//! and decoding BlurHashes. It minimizes the number of allocated arrays to reduce
//! the memory footprint. The base83 encode and decode are also both very fast!
//!
//! #### Example
//!
//! Generating a blurhash from an image:
//! ```no_run
//! use fast_blurhash::compute_dct;
//!
//! let (width, height) = todo!("Get image width and height");
//! let image: Vec<u32> = todo!("Load the image");
//! let blurhash = compute_dct(&image, width, height, 3, 4).into_blurhash();
//! ```
//!
//! Generating an image from a blurhash:
//! ```
//! use fast_blurhash::decode;
//!
//! let blurhash = "LlMF%n00%#MwS|WCWEM{R*bbWBbH";
//! let image: Vec<u32> = decode(&blurhash, 1.).unwrap().to_rgba(32, 32);
//! ```
//!
//! ## Custom color types
//!
//! **fast-blurhash** provides an easy way to convert custom types for pixel values
//! into the linear space to be used by the algorithm. Simply implements the trait
//! `AsLinear` on your type!
//!
//! #### Example
//!
//! ```no_run
//! use fast_blurhash::{convert::{AsLinear, Linear, srgb_to_linear}, compute_dct};
//!
//! struct MyColor {
//!     r: u8,
//!     g: u8,
//!     b: u8
//! }
//!
//! impl AsLinear for MyColor {
//!     fn as_linear(&self) -> Linear {
//!         [srgb_to_linear(self.r), srgb_to_linear(self.g), srgb_to_linear(self.b)]
//!     }
//! }
//!
//! // And then compute the blurhash!
//! let (width, height) = todo!("Get image width and height");
//! let image: Vec<MyColor> = todo!("Load the image");
//! let blurhash = compute_dct(&image, width, height, 3, 4).into_blurhash();
//! ```
//!
//! Several conversion function are available such as sRGB to Linear, check out the
//! [`convert`] module.
//!
//! [`convert`]: convert/index.html
//!
//! You can also generate an image using your custom type:
//! ```
//! use fast_blurhash::{decode, convert::linear_to_srgb};
//!
//! struct MyColor {
//!     r: u8,
//!     g: u8,
//!     b: u8
//! }
//!
//! let blurhash = "LlMF%n00%#MwS|WCWEM{R*bbWBbH";
//! let image: Vec<MyColor> = decode(&blurhash, 1.).unwrap().to_image(32, 32, |l| MyColor {
//!     r: linear_to_srgb(l[0]),
//!     g: linear_to_srgb(l[1]),
//!     b: linear_to_srgb(l[2])
//! });
//! ```
//!
//! ## Using iterators
//!
//! You can also use the iterator version of the *compute_dct* function to
//! prevent allocating more memory for the type conversion. This is espacially
//! useful with nested types. Plus it has no performance overhead.
//! However, make sure the iterator is long enough or the result of the DCT will
//! be incorrect.
//!
//! #### Example
//! ```no_run
//! use fast_blurhash::{convert::{AsLinear, Linear, srgb_to_linear}, compute_dct_iter};
//!
//! struct Color(u8, u8, u8);
//!
//! impl AsLinear for &Color {
//!     fn as_linear(&self) -> Linear {
//!         [srgb_to_linear(self.0), srgb_to_linear(self.1), srgb_to_linear(self.2)]
//!     }
//! }
//!
//! // And then compute the blurhash!
//! let (width, height) = todo!("Get image width and height");
//! let image: Vec<Vec<Color>> = todo!("Load the image");
//! let blurhash = compute_dct_iter(image.iter().flatten(), width, height, 3, 4).into_blurhash();
//! ```

pub mod base83;
pub mod convert;

use std::f32::consts::PI;
use convert::*;
use base83::encode_fixed_to;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlurhashError {
    /// Occurs when the provided blurhash's lenght is not the same as the expected
    /// length that was extracted from the 'header' (first char) of the blurhash
    InvalidLength,
    /// Occurs when the provided punch is not a non-zero positive float.
    InvalidPunch,
    /// The blurhash contains invalid base83 codes.
    BadFormat(base83::Base83ConversionError),
    /// The blurhash's number of X or Y components was greater than 9.
    UnsupportedMode,
}

impl std::fmt::Display for BlurhashError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use BlurhashError::*;
        match self {
            InvalidLength => write!(fmt, "The extracted length of the blurhash does not match the actual length"),
            InvalidPunch => write!(fmt, "The punch parameter must be positive and non-zero"),
            BadFormat(kind) => write!(fmt, "The blurhash contains invalid base83 codes ({kind:?})"),
            UnsupportedMode => write!(fmt, "The blurhash's number of X or Y components was greater than 9")
        }
    }
}

impl From<base83::Base83ConversionError> for BlurhashError {
    fn from(err: base83::Base83ConversionError) -> Self {
        Self::BadFormat(err)
    }
}

/// DCTResult is the result of a Discrete Cosine Transform performed on a image
/// with a specific number of X and Y components. It stores the frequency and
/// location of colors within the image.
#[derive(Default, Clone, Debug)]
pub struct DCTResult {
    /// The absolute maximum value of each channel in the alternative currents
    ac_max: f32,
    /// 2D-array represented in row-major column (x_components columns and
    /// y_components rows) that stores information about the average color in
    /// the cosine distribution (kernel).
    /// The totak number of currents is (x_components * y_components)
    currents: Vec<Factor>,
    /// Number of X components
    x_components: usize,
    /// Number of Y components
    y_components: usize
}

impl DCTResult {
    /// Store the result of a DCT
    pub fn new(ac_max: f32, currents: Vec<Factor>, x_components: usize, y_components: usize) -> DCTResult {
        assert!(currents.len() == x_components * y_components);
        assert!(ac_max != 0.);

        DCTResult { ac_max, currents, x_components, y_components }
    }

    /// Convert the computed color frequencies into a base83 string using
    /// the wolt/blurhash algorithm.
    pub fn into_blurhash(self) -> String {
        encode(&self)
    }

    /// Generate an image from this DCT Result to recreate (sort of) the original
    /// image. This function allocates a vector of (width * height) pixels in
    /// the linear space.
    pub fn to_image<T>(&self, width: usize, height: usize, convert: fn(Linear) -> T) -> Vec<T> {
        let mut pixels = Vec::with_capacity(width * height);

        for y in 0..height {
            let percent_y = y as f32 / height as f32;
            for x in 0..width {
                let percent_x = x as f32 / width as f32;

                let mut col = inv_multiply_basis(self.x_components, self.y_components,
                    percent_x, percent_y, &self.currents);

                col[0] = col[0].max(0.).min(1.);
                col[1] = col[1].max(0.).min(1.);
                col[2] = col[2].max(0.).min(1.);

                pixels.push(convert(col));
            }
        }

        pixels
    }

    /// Generate an image from this DCT Result to recreate (sort of) the original
    /// image. This function allocates a vector of (width * height) pixels in
    /// the sRGB space as in [RR, GG, BB].
    pub fn to_rgb8(&self, width: usize, height: usize) -> Vec<[u8; 3]> {
        self.to_image(width, height, |col| [
            linear_to_srgb(col[0]),
            linear_to_srgb(col[1]),
            linear_to_srgb(col[2]),
        ])
    }

    /// Generate an image from this DCT Result to recreate (sort of) the original
    /// image. This function allocates a vector of (width * height) pixels in
    /// the sRGB space as in [RR, GG, BB, AA]. (alpha will always be 255).
    pub fn to_rgba8(&self, width: usize, height: usize) -> Vec<[u8; 4]> {
        self.to_image(width, height, |col| [
            linear_to_srgb(col[0]),
            linear_to_srgb(col[1]),
            linear_to_srgb(col[2]),
            255
        ])
    }

    /// Generate an image from this DCT Result to recreate (sort of) the original
    /// image. This function allocates a vector of (width * height) u32 in
    /// the sRGB space as in AARRGGBB in hex (alpha will always be 255).
    pub fn to_rgba(&self, width: usize, height: usize) -> Vec<u32> {
        self.to_image(width, height, |col|
            ((linear_to_srgb(col[2]) as u32) <<  0) |
            ((linear_to_srgb(col[1]) as u32) <<  8) |
            ((linear_to_srgb(col[0]) as u32) << 16) |
            ((255                    as u32) << 24)
        )
    }

    /// Retrieve the currents of the DCT. The returned array is
    /// a 2D-array represented in row-major column with
    /// (x_components * y_components) items. Note that the first current is the
    /// DC or Direct Current.
    pub fn currents(&self) -> &[Factor] {
        &self.currents
    }

    /// Retrieve the ACs or Alternative Currents of the DCT. The returned array is
    /// a 2D-array represented in row-major column with (x_components * y_components) - 1 items.
    /// Note that the first current, which is the DC or Direct Current, is not included.
    pub fn acs(&self) -> &[Factor] {
        &self.currents[1..]
    }

    /// Retrieve the Direct Current or DC of the DCT. It corresponds to the
    /// average value of the components. For example, in an image it would be
    /// the average color of the image.
    pub fn dc(&self) -> &Factor {
        &self.currents[0]
    }

    /// Retrive the number of X components
    pub fn x_components(&self) -> usize {
        self.x_components
    }

    /// Retrive the number of Y components
    pub fn y_components(&self) -> usize {
        self.y_components
    }

    // Retrive the dimension (x_components, y_components) of the computed DCT
    pub fn dim(&self) -> (usize, usize) {
        (self.x_components, self.y_components)
    }
}

/// Compute the blurhash string from the DCT result using the wolt/blurhash format.
/// This function allocates a string of length (1 + 1 + 4 + 2 * components) where
/// components is the total number of components (components_x * components_y).
pub fn encode(dct: &DCTResult) -> String {
    let DCTResult { mut ac_max, currents, x_components, y_components } = dct;
    assert!((1..=9).contains(x_components), "The number of X components must be between 1 and 9");
    assert!((1..=9).contains(y_components), "The number of Y components must be between 1 and 9");

    let mut blurhash = String::with_capacity(1 + 1 + 4 + 2 * (currents.len() - 1));

    encode_fixed_to(((x_components - 1) + (y_components - 1) * 9) as u32, 1, &mut blurhash);

    if currents.len() > 0 {
        let quantised_max = (ac_max * 166. - 0.5).floor().min(82.).max(0.);
        encode_fixed_to(quantised_max as u32, 1, &mut blurhash);
        ac_max = (quantised_max + 1.) / 166.;
    } else {
        encode_fixed_to(0, 1, &mut blurhash);
    }

    encode_fixed_to(to_rgb(currents[0]), 4, &mut blurhash);

    for &ac in currents.iter().skip(1) {
        encode_fixed_to(encode_ac(ac, ac_max), 2, &mut blurhash);
    }

    blurhash
}

/// Decode a blurhash to retrive the DCT results (containing the color frequencies
/// disposition) using the wolt/blurhash format. This function may allocate up to a
/// vector of length 81 contained in the DCTResult struct.
pub fn decode(blurhash: &str, punch: f32) -> Result<DCTResult, BlurhashError> {
    if punch <= 0. {
        return Err(BlurhashError::InvalidPunch)
    }

    if blurhash.is_empty() {
        return Err(BlurhashError::InvalidLength)
    }
    let total = base83::decode(&blurhash[..1])? as usize;
    let (x_components, y_components) = ((total % 9) + 1, (total / 9) + 1);

    if x_components > 9 || y_components > 9 {
        return Err(BlurhashError::UnsupportedMode)
    }

    let current_count = x_components * y_components;
    if blurhash.len() != 1 + 1 + 4 + 2 * (current_count - 1) {
        return Err(BlurhashError::InvalidLength)
    }

    let ac_max = base83::decode(&blurhash[1..2])? + 1;
    let ac_max = ((ac_max as f32) / 166.) * punch;

    let mut currents = Vec::with_capacity(current_count);
    currents.push(decode_dc(base83::decode(&blurhash[2..6])?));

    for i in 1..current_count {
        let idx = (i - 1) * 2 + 6;
        let ac = base83::decode(&blurhash[idx..(idx + 2)])?;
        currents.push(decode_ac(ac, ac_max));
    }

    Ok(DCTResult { ac_max, currents, x_components, y_components })
}

/// Compute the Discrete Cosine Transform on an image in linear space. The iterator
/// must be long enough (it must have at least width * height items).
///
/// Altought the function traverses only once the input image and allocates a
/// vector of (x_components * y_components * 3) floats, the process may be a
/// little slow depending on the size of the input image.
///
/// Note: To generate a valid blurhash, the number of X or/and Y components
/// must be between 1 and 9. This is a limitation of the encoding scheme.
pub fn compute_dct_iter<T: AsLinear>(image: impl Iterator<Item = T>, width: usize, height: usize, x_components: usize, y_components: usize) -> DCTResult {
    let mut currents: Vec<Factor> = vec![[0., 0., 0.]; x_components * y_components];

    let total = width * height;
    for (i, pixel) in image.take(total).enumerate() {
        let col = pixel.as_linear();

        let p = i as f32 / width as f32;
        let percent_y = p / height as f32;
        let percent_x = p.fract();

        multiply_basis(x_components, y_components, percent_x, percent_y, &col, &mut currents);
    }

    let ac_max = normalize_and_max(&mut currents, total);

    DCTResult { ac_max, currents, x_components, y_components }
}

/// Compute the Discrete Cosine Transform on an image in linear space. The slice
/// must be long enough (it must have at least width * height items).
///
/// Altought the function traverses only once the input image and allocates a
/// vector of (x_components * y_components * 3) floats, the process may be a
/// little slow depending on the size of the input image.
///
/// Note: To generate a valid blurhash, the number of X or/and Y components
/// must be between 1 and 9. This is a limitation of the encoding scheme.
pub fn compute_dct<T: AsLinear>(image: &[T], width: usize, height: usize, x_components: usize, y_components: usize) -> DCTResult {
    assert!(image.len() >= width * height);
    let mut currents: Vec<Factor> = vec![[0., 0., 0.]; x_components * y_components];

    for y in 0..height {
        let percent_y = y as f32 / height as f32;
        for x in 0..width {
            let percent_x = x as f32 / width as f32;

            let col = image[y * width + x].as_linear();
            multiply_basis(x_components, y_components, percent_x, percent_y, &col, &mut currents);
        }
    }

    let ac_max = normalize_and_max(&mut currents, width * height);

    DCTResult { ac_max, currents, x_components, y_components }
}

/// Compute an iteration of the DCT for every component on the pixel (x, y)
/// that have the color `col` in linear space. Note that the currents slice must
/// be long enough (x_comps * y_comps) and the pixel coordinates (x, y) are between
/// 0 and 1.
#[inline]
pub fn multiply_basis(x_comps: usize, y_comps: usize, x: f32, y: f32, col: &[f32; 3], currents: &mut [Factor]) {
    for comp_y in 0..y_comps {
        let base_y = (PI * comp_y as f32 * y).cos();

        for comp_x in 0..x_comps {
            let f = &mut currents[comp_y * x_comps + comp_x];

            let base_x = (PI * comp_x as f32 * x).cos();
            let basis = base_y * base_x;

            f[0] += basis * col[0];
            f[1] += basis * col[1];
            f[2] += basis * col[2];
        }
    }
}

/// Compute an iteration of the inverse DCT for every component on the pixel (x, y)
/// and stores the color of that pixel into `col`. Note that the currents slice must
/// be long enough (x_comps * y_comps).
#[inline]
pub fn inv_multiply_basis(x_comps: usize, y_comps: usize, x: f32, y: f32, currents: &[Factor]) -> [f32; 3] {
    let mut col = [0.; 3];
    for comp_y in 0..y_comps {
        let base_y = (PI * comp_y as f32 * y).cos();

        for comp_x in 0..x_comps {
            let f = currents[comp_y * x_comps + comp_x];

            let base_x = (PI * comp_x as f32 * x).cos();
            let basis = base_y * base_x;

            col[0] += basis * f[0];
            col[1] += basis * f[1];
            col[2] += basis * f[2];
        }
    }

    col
}

/// Normalizes in-plae the currents by a predefined quantization table for the
/// wolt/blurhash encoding algorithm (1 for DC and 2 for ACs) and returns the
/// absolute maximum value within every channel of every currents. Note that
/// currents must have one or more items and len is the total number of pixels
/// of the image (width * height).
pub fn normalize_and_max(currents: &mut [Factor], len: usize) -> f32 {
    let len = len as f32;
    let norm = 1. / len; // Normalisation for DC is 1
    let f = &mut currents[0];
    f[0] *= norm;
    f[1] *= norm;
    f[2] *= norm;

    if currents.len() == 1 {
        return 1.
    }

    let mut ac_max = 0f32;
    let norm = 2. / len; // Normalisation for ACs is 2
    for f in currents.iter_mut().skip(1).flatten() {
        *f *= norm;
        ac_max = ac_max.max(f.abs());
    }

    ac_max
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiply_basis() {
        let width: usize = 4;
        let height: usize = 4;
        let x_comps: usize = 5;
        let y_comps: usize = 5;
        let image: [Linear; 16] = [
            [1., 1., 1.], [0., 0., 0.], [1., 1., 1.], [0., 0., 0.],
            [0., 0., 0.], [0., 0., 0.], [1., 1., 1.], [0., 0., 0.],
            [1., 1., 1.], [1., 1., 1.], [1., 1., 1.], [1., 1., 1.],
            [0., 0., 0.], [0., 0., 0.], [1., 1., 1.], [0., 0., 0.],
        ];
        let mut currents: Vec<Factor> = vec![[0., 0., 0.]; x_comps * y_comps];

        for y in 0..height {
            let percent_y = y as f32 / height as f32;
            for x in 0..width {
                let percent_x = x as f32 / width as f32;
                multiply_basis(x_comps, y_comps, percent_x, percent_y,
                    &image[y * width + x], &mut currents);
            }
        }

        let average_color = [8., 8., 8.]; // 8/16 of the colors are black
        assert_eq!(currents[0 * x_comps + 0], average_color);

        // the (0, 2) kernel looks like this:
        // [   1,   1,   1,   1,
        //    ~0,  ~0,  ~0,  ~0,
        //    -1,  -1,  -1,  -1,
        //    ~0,  ~0,  ~0,  ~0  ]
        // Image * kernel looks like this:
        // [  1,   .,   1,   .,
        //    .,   .,   .,   .,
        //   -1,  -1,  -1,  -1,
        //    .,   .,   .,   .  ] => adding up to -2
        assert_eq!(currents[2 * x_comps + 0], [-2., -2., -2.]);

        // the (2, 0) kernel looks like this:
        // [  1,  ~0, -1,  ~0,
        //    1,  ~0, -1,  ~0,
        //    1,  ~0, -1,  ~0,
        //    1,  ~0, -1,  ~0  ]
        // Image * kernel looks like this:
        // [  1,   .,  -1,  .,
        //    .,   .,  -1,  .,
        //    1,   .,  -1,  .,
        //    .,   .,  -1,  .  ] => adding up to -2
        assert_eq!(currents[0 * x_comps + 2], [-2., -2., -2.]);

        // the (3, 3) kernel looks like this:
        // [     1,  -0.7,  ~0,   0.7,
        //    -0.7,   0.x_comps,  ~0,  -0.x_comps,
        //      ~0,    ~0,  ~0,    ~0,
        //     0.7,  -0.x_comps,  ~0,  -0.x_comps  ]
        // Image * kernel looks like this:
        // [  1,   .,   .,   .,
        //    .,   .,   .,   .,
        //    .,   .,   .,   .,
        //    .,   .,   .,   .  ] => adding up to 1
        assert_eq!(currents[3 * x_comps + 3], [1., 1., 1.]);

        // the (4, 2) kernel looks like this:
        // [  1,  -1,   1,  -1,
        //   ~0,  ~0,  ~0,  ~0,
        //    1,  -1,   1,  -1,
        //   ~0,  ~0,  ~0,  ~0  ]
        // Image * kernel looks like this:
        // [  1,   .,   1,   .,
        //    .,   .,   .,   .,
        //    1,  -1,   1,  -1,
        //    .,   .,   .,   .  ] => adding up to 2
        assert_eq!(currents[2 * x_comps + 4], [2., 2., 2.]);

        // the (2, 4) kernel looks like this:
        // [  1,  ~0,  -1,  ~0,
        //   -1,  ~0,   1,  ~0,
        //    1,  ~0,  -1,  ~0,
        //   -1,  ~0,   1,  ~0  ]
        // Image * kernel looks like this:
        // [  1,   .,   1,   .,
        //    .,   .,  -1,   .,
        //    1,   .,   1,   .,
        //    .,   .,  -1,   .  ] => adding up to 2
        assert_eq!(currents[4 * x_comps + 2], [2., 2., 2.]);
    }

    #[test]
    fn test_encode_33() {
        let image: [Rgb; 16] = [
            [255,   0,   0], [  0,   0,   0], [255, 255, 255], [  0,   0,   0],
            [  0,   0,   0], [  0,   0,   0], [255, 255, 255], [  0,   0,   0],
            [255, 255, 255], [255, 255, 255], [  0, 255,   0], [255, 255, 255],
            [  0,   0,   0], [  0,   0,   0], [255, 255, 255], [  0,   0,   0],
        ];
        assert_eq!(compute_dct(&image, 4, 4, 3, 3).into_blurhash(), "KzKUZY=|HZ=|$5e9HZe9IS");
    }

    #[test]
    fn test_encode_decode_no_comps() {
        let image: [Rgb; 16] = [[255, 127, 55]; 16];
        let dct = compute_dct(&image, 4, 4, 1, 1);
        let blurhash = encode(&dct);
        assert_eq!(blurhash, "0~TNl]");

        let inv = decode(&blurhash, 1.).unwrap();
        assert_eq!(inv.x_components, dct.x_components);
        assert_eq!(inv.y_components, dct.y_components);

        for (i, (a, b)) in inv.currents.iter().flatten().zip(dct.currents.iter().flatten()).enumerate() {
            assert!((a - b).abs() < 0.05, "{a}, {b}: Error too big at index {i}");
        }
    }

    #[test]
    fn test_encode_decode_white() {
        let image: [Rgb; 16] = [[255, 255, 255]; 16];
        let dct = compute_dct(&image, 4, 4, 4, 4);
        let blurhash = encode(&dct);
        assert_eq!(blurhash, "U~TSUA~qfQ~q~q%MfQ%MfQfQfQfQ~q%MfQ%M");
        let inv = decode(&blurhash, 1.).unwrap();
        assert_eq!(inv.x_components, dct.x_components);
        assert_eq!(inv.y_components, dct.y_components);

        for (i, (a, b)) in inv.currents.iter().flatten().zip(dct.currents.iter().flatten()).enumerate() {
            assert!((a - b).abs() < 0.05, "{a}, {b}: Error too big at index {i}");
        }

        let generated = inv.to_image(4, 4, |p| p);
        println!("{generated:?}");
        for (i, &pixel) in generated.iter().flatten().enumerate() {
            assert!((pixel - 1.).abs() < 0.05, "Expected white pixel got {pixel} at index {i}");
        }
    }

    #[test]
    fn test_encode_decode_black() {
        let image: [Rgb; 16] = [[0, 0, 0]; 16];
        let dct = compute_dct(&image, 4, 4, 4, 4);
        let blurhash = encode(&dct);
        assert_eq!(blurhash, "U00000fQfQfQfQfQfQfQfQfQfQfQfQfQfQfQ");

        let inv = decode(&blurhash, 1.).unwrap();
        assert_eq!(inv.x_components, dct.x_components);
        assert_eq!(inv.y_components, dct.y_components);
        assert_eq!(inv.dc(), dct.dc());

        for (i, (a, b)) in inv.currents.iter().flatten().zip(dct.currents.iter().flatten()).enumerate() {
            assert!((a - b).abs() < 0.05, "{a}, {b} at index {i}");
        }

        let generated = inv.to_image(4, 4, |p| p);
        for (i, &pixel) in generated.iter().flatten().enumerate() {
            assert!((pixel - 0.).abs() < 0.05, "Expected black pixel got {pixel} at index {i}");
        }
    }

    use ril::prelude::Image;

    impl AsLinear for &ril::pixel::Rgb {
        fn as_linear(&self) -> Linear {
            [srgb_to_linear(self.r), srgb_to_linear(self.g), srgb_to_linear(self.b)]
        }
    }

    #[test]
    fn test_encode_image() {
        let img = Image::<ril::pixel::Rgb>::open("test.webp").unwrap();
        let w = img.width() as usize;
        let h = img.height() as usize;
        let s = compute_dct_iter(img.pixels().flatten(), w, h, 4, 7);
        assert_eq!(s.into_blurhash(), "vbHCG?SgNGxD~pX9R+i_NfNIt7V@NL%Mt7Rj-;t7e:WCfPWXV[ofM{WXbHof");
    }

    #[test]
    fn test_decode_image() {
        let s = decode("vbHLxdSgNHxD~pX9R+i_NfNIt7V@NL%Mt7Rj-;t7e:WCj[WXV[ofM{WXbHof", 1.)
            .unwrap().to_rgb8(32, 48);

        let img = Image::<ril::pixel::Rgb>::from_fn(32, 48, |x, y| {
            let [r, g, b] = s[y as usize * 32 + x as usize];
            ril::pixel::Rgb::new(r, g, b)
        });
        img.save_inferred("out.webp").unwrap();
    }
}
