//! # Fast(er) BlurHash
//!
//! Provides a faster implementation of the BlurHash algorithm for encoding
//! and decoding BlurHashes. It minimizes the number of allocated arrays to reduce
//! the memory footprint. The base83 encode and decode are also both very fast!
//!
//! #### Example
//!
//! ```no_run
//! use fast_blurhash::compute_dct;
//!
//! let (width, height) = todo!("Get image width and height");
//! let image: Vec<u32> = todo!("Load the image");
//! let blurhash = compute_dct(&image, width, height, 3, 4).to_blurhash();
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
//! use fast_blurhash::{AsLinear, Linear, compute_dct};
//!
//! struct MyColor {
//!     r: u8,
//!     g: u8,
//!     b: u8
//! }
//!
//! impl AsLinear for MyColor {
//!     fn as_linear(&self) -> Linear {
//!         use fast_blurhash::convert::srgb_to_linear;
//!         [srgb_to_linear(self.r), srgb_to_linear(self.g), srgb_to_linear(self.b)]
//!     }
//! }
//!
//! // And then compute the blurhash!
//! let (width, height) = todo!("Get image width and height");
//! let image: Vec<MyColor> = todo!("Load the image");
//! let blurhash = compute_dct(&image, width, height, 3, 4).to_blurhash();
//! ```
//!
//! Several conversion function are available such as sRGB to Linear, check out the
//! [`convert`] module.
//!
//! [`convert`]: convert/index.html
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
//! use fast_blurhash::{AsLinear, Linear, compute_dct_iter};
//!
//! struct Color(u8, u8, u8);
//!
//! impl AsLinear for &Color {
//!     fn as_linear(&self) -> Linear {
//!         use fast_blurhash::convert::srgb_to_linear;
//!         [srgb_to_linear(self.0), srgb_to_linear(self.1), srgb_to_linear(self.2)]
//!     }
//! }
//!
//! // And then compute the blurhash!
//! let (width, height) = todo!("Get image width and height");
//! let image: Vec<Vec<Color>> = todo!("Load the image");
//! let blurhash = compute_dct_iter(image.iter().flatten(), width, height, 3, 4).to_blurhash();
//! ```

pub mod base83;
pub mod convert;

use std::f32::consts::PI;
use convert::*;
use base83::encode_fixed_to;

/// RGB Color in the linear space
pub type Linear = [f32; 3];
/// RGB Frequencies of a specific cosine transform
pub type Factor = [f32; 3];
/// RGB 8-bit per channel color
pub type Rgb = [u8; 3];

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
    pub fn to_blurhash(self) -> String {
        encode(self)
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
    pub fn x_components(&self) {
        self.x_components
    }

    /// Retrive the number of Y components
    pub fn y_components(&self) {
        self.y_components
    }

    // Retrive the dimension (x_components, y_components) of the computed DCT
    pub fn dim(&self) -> (usize, usize) {
        (self.x_components, self.y_components)
    }
}

/// Converts any kind of Color to the linear space to be used in with DCT
pub trait AsLinear {
    /// Returns the color represented in linear space.
    fn as_linear(&self) -> Linear;
}

impl AsLinear for [u8; 3] {
    fn as_linear(&self) -> Linear {
        [srgb_to_linear(self[0]), srgb_to_linear(self[1]), srgb_to_linear(self[2])]
    }
}

impl AsLinear for &[u8; 3] {
    fn as_linear(&self) -> Linear {
        [srgb_to_linear(self[0]), srgb_to_linear(self[1]), srgb_to_linear(self[2])]
    }
}

impl AsLinear for [u8; 4] {
    fn as_linear(&self) -> Linear {
        [srgb_to_linear(self[0]), srgb_to_linear(self[1]), srgb_to_linear(self[2])]
    }
}

impl AsLinear for &[u8; 4] {
    fn as_linear(&self) -> Linear {
        [srgb_to_linear(self[0]), srgb_to_linear(self[1]), srgb_to_linear(self[2])]
    }
}

impl AsLinear for u32 {
    fn as_linear(&self) -> Linear {
        [srgb_to_linear(((self >> 16) & 0xFF) as u8), // red
         srgb_to_linear(((self >>  8) & 0xFF) as u8), // green
         srgb_to_linear(((self >>  0) & 0xFF) as u8)] // blue
    }
}

/// Compute the blurhash string from the DCT result using the wolt/blurhash algorithm.
/// This function allocates a string of length (1 + 1 + 4 + 2 * components) where
/// components is the total number of components (components_x * components_y).
pub fn encode(dct: DCTResult) -> String {
    let DCTResult { mut ac_max, currents, x_components, y_components } = dct;
    assert!((1..=9).contains(&x_components), "The number of X components must be between 1 and 9");
    assert!((1..=9).contains(&y_components), "The number of Y components must be between 1 and 9");

    let mut blurhash = String::with_capacity(1 + 1 + 4 + 2 * (currents.len() - 1));

    encode_fixed_to(((x_components - 1) + (y_components - 1) * 9) as u32, 1, &mut blurhash);

    if currents.len() > 0 {
        let quantised_max = (ac_max * 166. - 0.5).floor().min(82.).max(0.);
        encode_fixed_to(quantised_max as u32, 1, &mut blurhash);
        ac_max = (quantised_max + 1.) / 166.;
    } else {
        encode_fixed_to(0, 1, &mut blurhash);
    }

    encode_fixed_to(encode_dc(currents[0]), 4, &mut blurhash);

    for ac in currents.into_iter().skip(1) {
        encode_fixed_to(encode_ac(ac, ac_max), 2, &mut blurhash);
    }

    blurhash
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

    let ac_max = if currents.len() > 1 {
        normalize_and_max(&mut currents, total)
    } else {
        1.
    };

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
    let mut currents: Vec<Factor> = vec![[0., 0., 0.]; x_components * y_components];

    for y in 0..height {
        let percent_y = y as f32 / height as f32;
        for x in 0..width {
            let percent_x = x as f32 / width as f32;

            let col = image[y * width + x].as_linear();
            multiply_basis(x_components, y_components, percent_x, percent_y, &col, &mut currents);
        }
    }

    let ac_max = if currents.len() > 1 {
        normalize_and_max(&mut currents, width * height)
    } else {
        1.
    };

    DCTResult { ac_max, currents, x_components, y_components }
}

/// Compute an iteration of the DCT for every component on the pixel (x, y)
/// that have the color `col` in linear space. Note that the currents slice must
/// be long enough (x_comps * y_comps).
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
        assert_eq!(compute_dct(&image, 4, 4, 3, 3).to_blurhash(), "KzKUZY=|HZ=|$5e9HZe9IS");
    }

    #[test]
    fn test_encode_white() {
        let image: [Rgb; 16] = [[255, 255, 255]; 16];
        assert_eq!(compute_dct(&image, 4, 4, 4, 4).to_blurhash(), "U~TSUA~qfQ~q~q%MfQ%MfQfQfQfQ~q%MfQ%M");
    }

    #[test]
    fn test_encode_black() {
        let image: [Rgb; 16] = [[0, 0, 0]; 16];
        assert_eq!(compute_dct(&image, 4, 4, 4, 4).to_blurhash(), "U00000fQfQfQfQfQfQfQfQfQfQfQfQfQfQfQ");
    }

    /*
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
        assert_eq!(s.to_blurhash(), "vbHLxdSgNHxD~pX9R+i_NfNIt7V@NL%Mt7Rj-;t7e:WCj[WXV[ofM{WXbHof");
    }*/
}
