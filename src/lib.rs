pub mod base83;
pub mod convert;

use std::f32::consts::PI;
use convert::*;
use base83::encode_fixed_to;

pub type Linear = [f32; 3];
pub type Factor = [f32; 3];
pub type Rgb = [u8; 3];

#[derive(Default, Clone, Debug)]
pub struct DCTResult {
    pub ac_max: f32,
    pub acs: Vec<Factor>,
    pub x_components: u8,
    pub y_components: u8
}

impl DCTResult {
    pub fn to_blurhash(self) -> String {
        encode(self)
    }

    pub fn acs(&self) -> &[Factor] {
        &self.acs[1..]
    }

    pub fn dc(&self) -> &Factor {
        &self.acs[0]
    }
}

pub trait AsLinear {
    fn as_linear(&self) -> Linear;
}

impl AsLinear for Rgb {
    fn as_linear(&self) -> Linear {
        [srgb_to_linear(self[0]), srgb_to_linear(self[1]), srgb_to_linear(self[2])]
    }
}

impl AsLinear for &Rgb {
    fn as_linear(&self) -> Linear {
        [srgb_to_linear(self[0]), srgb_to_linear(self[1]), srgb_to_linear(self[2])]
    }
}

impl AsLinear for [u8; 4] {
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

pub fn encode(dct: DCTResult) -> String {
    let DCTResult { mut ac_max, acs, x_components, y_components } = dct;

    let mut blurhash = String::with_capacity(1 + 1 + 4 + 2 * (acs.len() - 1));

    encode_fixed_to(((x_components - 1) + (y_components - 1) * 9) as u32, 1, &mut blurhash);

    if acs.len() > 0 {
        let quantised_max = (ac_max * 166. - 0.5).floor().min(82.).max(0.);
        encode_fixed_to(quantised_max as u32, 1, &mut blurhash);
        ac_max = (quantised_max + 1.) / 166.;
    } else {
        encode_fixed_to(0, 1, &mut blurhash);
    }

    encode_fixed_to(encode_dc(acs[0]), 4, &mut blurhash);

    for ac in acs.into_iter().skip(1) {
        encode_fixed_to(encode_ac(ac, ac_max), 2, &mut blurhash);
    }

    blurhash
}

pub fn compute_dct_iter<T: AsLinear>(image: impl Iterator<Item = T>, width: usize, height: usize, x_components: usize, y_components: usize) -> DCTResult {
    debug_assert!((1..=9).contains(&x_components), "The number of X components must be between 1 and 9");
    debug_assert!((1..=9).contains(&y_components), "The number of Y components must be between 1 and 9");

    let mut acs: Vec<Factor> = vec![[0., 0., 0.]; x_components * y_components];

    for (i, pixel) in image.enumerate() {
        let col = pixel.as_linear();

        let p = i as f32 / width as f32;
        let percent_y = p / height as f32;
        let percent_x = p.fract();

        multiply_basis(x_components, y_components, percent_x, percent_y, &col, &mut acs);
    }

    let ac_max = if acs.len() > 1 {
        normalize_and_max(&mut acs, width * height)
    } else {
        1.
    };

    DCTResult {
        ac_max, acs,
        x_components: x_components as u8, y_components: y_components as u8
    }
}

pub fn compute_dct<T: AsLinear>(image: &[T], width: usize, height: usize, x_components: usize, y_components: usize) -> DCTResult {
    debug_assert!((1..=9).contains(&x_components), "The number of X components must be between 1 and 9");
    debug_assert!((1..=9).contains(&y_components), "The number of Y components must be between 1 and 9");

    let mut acs: Vec<Factor> = vec![[0., 0., 0.]; x_components * y_components];

    for y in 0..height {
        let percent_y = y as f32 / height as f32;
        for x in 0..width {
            let percent_x = x as f32 / width as f32;

            let col = image[y * width + x].as_linear();
            multiply_basis(x_components, y_components, percent_x, percent_y, &col, &mut acs);
        }
    }

    let ac_max = if acs.len() > 1 {
        normalize_and_max(&mut acs, width * height)
    } else {
        1.
    };

    DCTResult {
        ac_max, acs,
        x_components: x_components as u8, y_components: y_components as u8
    }
}

pub fn multiply_basis(x_comps: usize, y_comps: usize, x: f32, y: f32, col: &[f32; 3], acs: &mut [Factor]) {
    for comp_y in 0..y_comps {
        let base_y = (PI * comp_y as f32 * y).cos();

        for comp_x in 0..x_comps {
            let f = &mut acs[comp_y * x_comps + comp_x];

            let base_x = (PI * comp_x as f32 * x).cos();
            let basis = base_y * base_x;

            f[0] += basis * col[0];
            f[1] += basis * col[1];
            f[2] += basis * col[2];
        }
    }
}

pub fn normalize_and_max(acs: &mut [Factor], len: usize) -> f32 {
    let len = len as f32;
    let norm = 1. / len; // Normalisation for DC is 1
    let f = &mut acs[0];
    f[0] *= norm;
    f[1] *= norm;
    f[2] *= norm;

    let mut ac_max = 0f32;
    let norm = 2. / len; // Normalisation for ACs is 2
    for f in acs.iter_mut().skip(1).flatten() {
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
        let mut acs: Vec<Factor> = vec![[0., 0., 0.]; x_comps * y_comps];

        for y in 0..height {
            let percent_y = y as f32 / height as f32;
            for x in 0..width {
                let percent_x = x as f32 / width as f32;
                multiply_basis(x_comps, y_comps, percent_x, percent_y,
                    &image[y * width + x], &mut acs);
            }
        }

        let average_color = [8., 8., 8.]; // 8/16 of the colors are black
        assert_eq!(acs[0 * x_comps + 0], average_color);

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
        assert_eq!(acs[2 * x_comps + 0], [-2., -2., -2.]);

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
        assert_eq!(acs[0 * x_comps + 2], [-2., -2., -2.]);

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
        assert_eq!(acs[3 * x_comps + 3], [1., 1., 1.]);

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
        assert_eq!(acs[2 * x_comps + 4], [2., 2., 2.]);

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
        assert_eq!(acs[4 * x_comps + 2], [2., 2., 2.]);
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
