pub mod base83;
pub mod convert;

use std::f32::consts::PI;
use convert::*;
use base83::encode_to;

type Rgb = [u8; 3];
type Linear = [f32; 3];
type Factor = [f32; 3];

pub fn into_linear(image: Vec<Rgb>) -> Vec<Linear> {
    image.into_iter()
        .map(|rgb| [srgb_to_linear(rgb[0]), srgb_to_linear(rgb[1]), srgb_to_linear(rgb[2])])
        .collect()
}

fn to_linear(image: &[Rgb]) -> Vec<Linear> {
    image.iter()
        .map(|rgb| [srgb_to_linear(rgb[0]), srgb_to_linear(rgb[1]), srgb_to_linear(rgb[2])])
        .collect()
}

pub fn encode(image: &[Rgb], width: usize, height: usize, x_components: usize, y_components: usize) -> String {
    debug_assert!((1..=9).contains(&x_components), "The number of X components must be between 1 and 9");
    debug_assert!((1..=9).contains(&y_components), "The number of Y components must be between 1 and 9");

    let converted = to_linear(image); // copy
    encode_linear(&converted, width, height, x_components, y_components)
}

pub fn encode_linear(image: &[Linear], width: usize, height: usize, x_components: usize, y_components: usize) -> String {
    debug_assert!((1..=9).contains(&x_components), "The number of X components must be between 1 and 9");
    debug_assert!((1..=9).contains(&y_components), "The number of Y components must be between 1 and 9");

    let (dc, mut max, acs) = compute_components(image, width, height, x_components, y_components);

    let mut blurhash = String::with_capacity(1 + 1 + 4 + 2 * acs.len());

    encode_to(((x_components - 1) + (y_components - 1) * 9) as u32, &mut blurhash);

    if acs.len() > 0 {
        let quantised_max = (max * 166. - 0.5).floor().min(82.).max(0.);
        encode_to(quantised_max as u32, &mut blurhash);
        max = (quantised_max + 1.) / 166.;
    } else {
        encode_to(0, &mut blurhash);
    }

    encode_to(encode_dc(dc), &mut blurhash);

    for ac in acs.into_iter().map(|ac| encode_ac(ac, max)) {
        encode_to(ac, &mut blurhash);
    }

    blurhash
}

pub fn compute_components(image: &[Linear], width: usize, height: usize, x_components: usize, y_components: usize) -> (Factor, f32, Vec<Factor>) {
    // Calculate DC comoponent
    let norm = 1. / image.len() as f32; // Normalisation for DC is 1
    let dc = multiply_basis(0, 0, image, width, height, norm);

    let ac_count = x_components * y_components - 1;
    let mut acs: Vec<Factor> = Vec::with_capacity(ac_count);

    if ac_count == 0 {
        return (dc, 1., acs);
    }

    let s = std::time::Instant::now();
    // Calculate AC components
    let norm = 2. / image.len() as f32; // Normalisation for ACs is 2
    let mut ac_max = 0f32;
    for i in 1..=ac_count {
        let f = multiply_basis(i % x_components, i / x_components,
            image, width, height, norm);
        ac_max = ac_max.max(f[0].abs()).max(f[1].abs()).max(f[2].abs());
        acs.push(f);
    }
    println!("acs took {:?}", s.elapsed());

    (dc, ac_max, acs)
}

fn multiply_basis(comp_x: usize, comp_y: usize, image: &[Linear], width: usize, height: usize, norm: f32) -> Factor {
    let comp_x = comp_x as f32;
    let comp_y = comp_y as f32;

    let mut f: Factor = [0.; 3];
    for y in 0..height {
        let base_y = (PI * comp_y * y as f32 / height as f32).cos();
        for x in 0..width {
            let col = image[y * width + x];
            let base_x = (PI * comp_x * x as f32 / width as f32).cos();
            let basis = base_y * base_x;
            f[0] += basis * col[0];
            f[1] += basis * col[1];
            f[2] += basis * col[2];
        }
    }

    f[0] *= norm;
    f[1] *= norm;
    f[2] *= norm;

    f
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIDTH: usize = 4;
    const HEIGHT: usize = 4;
    const TEST_IMAGE: [Linear; 16] = [
        [1., 1., 1.], [0., 0., 0.], [1., 1., 1.], [0., 0., 0.],
        [0., 0., 0.], [0., 0., 0.], [1., 1., 1.], [0., 0., 0.],
        [1., 1., 1.], [1., 1., 1.], [1., 1., 1.], [1., 1., 1.],
        [0., 0., 0.], [0., 0., 0.], [1., 1., 1.], [0., 0., 0.],
    ];

    #[test]
    fn test_multiply_basis_ac() {
        let norm = 1. / TEST_IMAGE.len() as f32;
        let average_color = [1./2., 1./2., 1./2.]; // 8/16 of the colors are black
        assert_eq!(multiply_basis(0, 0, &TEST_IMAGE, WIDTH, HEIGHT, norm), average_color);
    }

    #[test]
    fn test_multiply_basis_dc02_20() {
        let norm = 2. / TEST_IMAGE.len() as f32;
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
        let v = -2. * norm;
        assert_eq!(multiply_basis(0, 2, &TEST_IMAGE, WIDTH, HEIGHT, norm), [v, v, v]);

        // the (0, 2) kernel looks like this:
        // [  1,  ~0, -1,  ~0,
        //    1,  ~0, -1,  ~0,
        //    1,  ~0, -1,  ~0,
        //    1,  ~0, -1,  ~0  ]
        // Image * kernel looks like this:
        // [  1,   .,  -1,  .,
        //    .,   .,  -1,  .,
        //    1,   .,  -1,  .,
        //    .,   .,  -1,  .  ] => adding up to -2
        assert_eq!(multiply_basis(2, 0, &TEST_IMAGE, WIDTH, HEIGHT, norm), [v, v, v]);
    }

    #[test]
    fn test_multiply_basis_dc33() {
        let norm = 2. / TEST_IMAGE.len() as f32;
        // the (3, 3) kernel looks like this:
        // [     1,  -0.7,  ~0,   0.7,
        //    -0.7,   0.5,  ~0,  -0.5,
        //      ~0,    ~0,  ~0,    ~0,
        //     0.7,  -0.5,  ~0,  -0.5  ]
        // Image * kernel looks like this:
        // [  1,   .,   .,   .,
        //    .,   .,   .,   .,
        //    .,   .,   .,   .,
        //    .,   .,   .,   .  ] => adding up to 1
        let v = 1. * norm;
        assert_eq!(multiply_basis(3, 3, &TEST_IMAGE, WIDTH, HEIGHT, norm), [v, v, v]);
    }

    #[test]
    fn test_multiply_basis_dc_42_24() {
        let norm = 2. / TEST_IMAGE.len() as f32;
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
        let v = 2. * norm;
        assert_eq!(multiply_basis(4, 2, &TEST_IMAGE, WIDTH, HEIGHT, norm), [v, v, v]);

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
        assert_eq!(multiply_basis(2, 4, &TEST_IMAGE, WIDTH, HEIGHT, norm), [v, v, v]);
    }

    const TEST_IMAGE_RGB: [Rgb; 16] = [
        [255,   0,   0], [  0,   0,   0], [255, 255, 255], [  0,   0,   0],
        [  0,   0,   0], [  0,   0,   0], [255, 255, 255], [  0,   0,   0],
        [255, 255, 255], [255, 255, 255], [  0, 255,   0], [255, 255, 255],
        [  0,   0,   0], [  0,   0,   0], [255, 255, 255], [  0,   0,   0],
    ];


    #[test]
    fn test_encode_33() {
        assert_eq!(encode(&TEST_IMAGE_RGB, WIDTH, HEIGHT, 3, 3), "KzKUZY=|HZ=|$5e9HZe9IS");
    }

    use ril::prelude::Image;
    use super::into_linear;

    #[test]
    fn test_encode_image() {
        let img = Image::<ril::pixel::Rgb>::open("test.webp").unwrap();
        let w = img.width() as usize;
        let h = img.height() as usize;
        let start = std::time::Instant::now();
        let pixels: Vec<Linear> = img.pixels().flatten()
            .map(|p| [srgb_to_linear(p.r), srgb_to_linear(p.g), srgb_to_linear(p.b)]).collect();
        println!("convert took {:?}", start.elapsed());
        let start = std::time::Instant::now();
        let s = encode_linear(&pixels, w, h, 4, 7);
        println!("encode_linear took {:?}", start.elapsed());
        assert_eq!(s, "vbHLxdSgNHxD~pX9R+i_NfNIt7V@NL%Mt7Rj-;t7e:WCj[WXV[ofM{WXbHof");
    }
}
