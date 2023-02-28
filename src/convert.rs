pub fn linear_to_srgb(linear: f32) -> u8 {
    let linear = linear.max(0.).min(1.);
    if linear <= 0.0031308 {
        (linear * 12.92 * 255. + 0.5).floor() as u8
    } else {
        ((1.055 * linear.powf(1. / 2.4) - 0.055) * 255. + 0.5).floor() as u8
    }
}

pub fn srgb_to_linear(pixel: u8) -> f32 {
    let normalized = pixel as f32 / 255.;
    if normalized <= 0.04045 {
        normalized / 12.92
    } else {
        ((normalized + 0.055) / 1.055).powf(2.4)
    }
}
