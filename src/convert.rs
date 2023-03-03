//! Color conversion and BlurHash specific encoding utilities

/// Convert a single channel in linear space to sRGB space
pub fn linear_to_srgb(linear: f32) -> u8 {
    let linear = linear.max(0.).min(1.);
    if linear <= 0.0031308 {
        (linear * 12.92 * 255. + 0.5).floor() as u8
    } else {
        ((1.055 * linear.powf(1. / 2.4) - 0.055) * 255. + 0.5).floor() as u8
    }
}

/// Convert a single channel in sRGB space to linear space
pub fn srgb_to_linear(pixel: u8) -> f32 {
    let normalized = pixel as f32 / 255.;
    if normalized <= 0.04045 {
        normalized / 12.92
    } else {
        ((normalized + 0.055) / 1.055).powf(2.4)
    }
}

/// Encodes a DC (average color) to an u32 to be encoded into a 4-digit base83
pub fn encode_dc(dc: [f32; 3]) -> u32 {
    let r = linear_to_srgb(dc[0]) as u32;
    let g = linear_to_srgb(dc[1]) as u32;
    let b = linear_to_srgb(dc[2]) as u32;
    (r << 16) | (g << 8) | b
}

/// Decodes a DC (average color) from an u32 to a color in linear space
pub fn decode_dc(n: u32) -> [f32; 3] {
    let r = ((n >> 16) & 0xFF) as u8;
    let g = ((n >>  8) & 0xFF) as u8;
    let b = ((n >>  0) & 0xFF) as u8;
    [srgb_to_linear(r), srgb_to_linear(g), srgb_to_linear(b)]
}

/// Calculates the 'absolute power' of a number.
/// If the number x is negative, it calculates `-(|x|)^exp`, `x^exp` otherwise.
pub fn sign_pow(x: f32, exp: f32) -> f32 {
    x.abs().powf(exp).copysign(x)
}

/// Encodes a AC to an u32 to be encoded into a 2-digit base83
pub fn encode_ac(ac: [f32; 3], ac_max: f32) -> u32 {
    let quant_r = (sign_pow(ac[0] / ac_max, 0.5) * 9. + 9.5).floor().min(18.).max(0.) as u32;
    let quant_g = (sign_pow(ac[1] / ac_max, 0.5) * 9. + 9.5).floor().min(18.).max(0.) as u32;
    let quant_b = (sign_pow(ac[2] / ac_max, 0.5) * 9. + 9.5).floor().min(18.).max(0.) as u32;

    quant_r * 19 * 19 + quant_g * 19 + quant_b
}

/// Decodes an AC from an u32
pub fn decode_ac(n: u32, ac_max: f32) -> [f32; 3] {
    let quant_r = (n / (19 * 19)) % 19;
    let quant_g = (n / 19) % 19;
    let quant_b = n % 19;

    [
        sign_pow((quant_r as f32 - 9.) / 9., 2.) * ac_max,
        sign_pow((quant_g as f32 - 9.) / 9., 2.) * ac_max,
        sign_pow((quant_b as f32 - 9.) / 9., 2.) * ac_max,
    ]
}
