pub mod base83;
pub mod convert;

use std::f32::consts::PI;
use convert::srgb_to_linear;

type Rgb = [u8; 3];
type Linear = [f32; 3];
type Factor = [f32; 3];

fn to_linear(image: &[Rgb]) -> Vec<Linear> {
    image.iter()
        .map(|rgb| [srgb_to_linear(rgb[0]), srgb_to_linear(rgb[1]), srgb_to_linear(rgb[2])])
        .collect()
}

pub fn encode(image: &[Rgb], width: usize, height: usize, x_components: usize, y_components: usize) -> String {
    assert!((1..=9).contains(&x_components), "The number of X components must be between 1 and 9");
    assert!((1..=9).contains(&y_components), "The number of Y components must be between 1 and 9");

    let converted = to_linear(image);
    let mut norm = 1. / image.len() as f32;

    let dc = multiply_basis(0, 0, &converted, width, height, norm);
    println!("dc: {dc:?}");

    let mut factors: Vec<Factor> = Vec::with_capacity(x_components * y_components - 1);
    norm *= 2.;

    for i in 0..factors.capacity() {
        let f = multiply_basis(i / y_components , i % x_components, &converted, width, height, norm);
        factors.push(f);
    }

    println!("factors: {factors:?}");

    todo!()
}

fn multiply_basis(comp_x: usize, comp_y: usize, image: &[Linear], width: usize, height: usize, norm: f32) -> Factor {
    let mut f: Factor = [0.; 3];

    let mut kernel = Vec::with_capacity(width * height);
    let comp_x = comp_x as f32;
    let comp_y = comp_y as f32;

    for y in 0..height {
        let base_y = (PI * comp_y * y as f32 / height as f32).cos();
        for x in 0..width {
            let base_x = (PI * comp_x * x as f32 / width as f32).cos();
            kernel.push(base_y * base_x);
        }
    }
    println!("kernel: {kernel:?}");

    for y in 0..height {
        for x in 0..width {
            let col = image[y * width + x];
            let basis = kernel[y * width + x];
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
}
