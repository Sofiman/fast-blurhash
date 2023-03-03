//! base83 encode and decode utilities

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Base83ConversionError {
    InvalidChar,
    Overflow
}

const CHARACTERS: [u8; 83] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z', b'#', b'$', b'%', b'*', b'+', b',', b'-', b'.', b':', b';', b'=', b'?', b'@', b'[', b']', b'^', b'_', b'{', b'|', b'}', b'~', 
];

/// Encodes an u32 to a base83 string. This function allocates a long-enough
/// string to contain the 1 to 6 base83 digit.
pub fn encode(mut n: u32) -> String {
    if n == 0 {
        return (CHARACTERS[0] as char).to_string();
    }

    let mut stack: [u8; 6] = [0; 6];
    let mut i = 0;

    while n > 0 {
        stack[i] = CHARACTERS[(n % 83) as usize];
        n /= 83;
        i += 1;
    }

    // allocate string
    let mut str = String::with_capacity(i);
    while i > 0 { // append to string in the reverse order
        i -= 1;
        str.push(stack[i] as char);
    }
    str
}

/// Encodes an u32 to a base83 string. This function does not allocate a string.
/// This function may append up to 6 new characters to the string.
pub fn encode_to(mut n: u32, str: &mut String) {
    if n == 0 {
        str.push(CHARACTERS[0] as char);
        return;
    }

    let mut stack: [u8; 6] = [0; 6];
    let mut i = 0;

    while n > 0 {
        stack[i] = CHARACTERS[(n % 83) as usize];
        n /= 83;
        i += 1;
    }

    while i > 0 { // append to string in the reverse order
        i -= 1;
        str.push(stack[i] as char);
    }
}

/// Encodes an u32 to a fixed size base83 string.
/// This function allocates a string of `iters` characters.
pub fn encode_fixed(mut n: u32, iters: u8) -> String {
    assert!(iters <= 6);
    let mut iters = iters as usize;

    let mut stack: [u8; 6] = [0; 6];

    for i in 0..iters {
        stack[i] = CHARACTERS[(n % 83) as usize];
        n /= 83;
    }

    // allocate string
    let mut str = String::with_capacity(iters);
    while iters > 0 { // append to string in the reverse order
        iters -= 1;
        str.push(stack[iters] as char);
    }
    str
}

/// Encodes an u32 to a fixed size base83 string. This function does not allocate
/// a string. This function appends `iters` new characters to the string.
pub fn encode_fixed_to(mut n: u32, iters: u8, str: &mut String) {
    assert!(iters <= 6);
    let mut iters = iters as usize;

    let mut stack: [u8; 6] = [0; 6];

    for i in 0..iters  {
        stack[i] = CHARACTERS[(n % 83) as usize];
        n /= 83;
    }

    while iters > 0 { // append to string in the reverse order
        iters -= 1;
        str.push(stack[iters] as char);
    }
}

const DIGITS: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, // 16

    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, // 32

    0, 0, 0, 62, 63, 64, 0, 0,
    0, 0, 65, 66, 67, 68, 69, 0, // 48

    0, 1, 2, 3, 4, 5, 6, 7,
    8, 9, 70, 71, 0, 72, 0, 73, // 64
    
    74, 10, 11, 12, 13, 14, 15, 16,
    17, 18, 19, 20, 21, 22, 23, 24, // 80
    
    25, 26, 27, 28, 29, 30, 31, 32,
    33, 34, 35, 75, 0, 76, 77, 78, // 96
    
    0, 36, 37, 38, 39, 40, 41, 42,
    43, 44, 45, 46, 47, 48, 49, 50, // 112
    51, 52, 53, 54, 55, 56, 57, 58,
    59, 60, 61, 79, 80, 81, 82, 0, // 128
   
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,

    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,

    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,

    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,

    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,

    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,

    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,

    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,

];

/// Decodes an base83-encoded ascii string to an u32. Note that this function
/// does not perform any runtime check on the input string, any ascii character
/// that is not part of the base83 character set.
pub fn decode_ascii(s: &str) -> u32 {
    debug_assert!(s.is_ascii());

    s.chars()
        .map(|c| DIGITS[c as usize] as u32)
        .fold(0, |acc, c| acc * 83 + c)
}

/// Decodes a base83-encoded string to an u32. This function returns None if the
/// string does not contain a valid u32 (in case of **non-ascii** characters or u32
/// overflow). Note that this function will ignore any ascii character that is not
/// part of the base83 character set.
pub fn decode(s: &str) -> Result<u32, Base83ConversionError> {
    let mut n: u32 = 0;

    let mut chars = s.chars();
    for _ in 0..5 { // no overflow until 6th character
        match chars.next() {
            Some(c) if c.is_ascii() => {
                n = n * 83 + DIGITS[c as usize] as u32;
            },
            Some(_) => return Err(Base83ConversionError::InvalidChar),
            None => return Ok(n) // end of string
        }
    }

    match chars.next() {
        Some(c) if c.is_ascii() => {
            n.checked_mul(83u32) // overflow check
                .ok_or(Base83ConversionError::Overflow)?
                .checked_add(DIGITS[c as usize] as u32)
                .ok_or(Base83ConversionError::Overflow)
        },
        Some(_) => Err(Base83ConversionError::InvalidChar), // invalid char
        None => Ok(n) // end of string
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! gen_enc_dec_test {
        (($name:ident, $val:expr, $base83:expr)) => {
            #[test]
            fn $name() {
                let mut s = String::with_capacity(6);
                encode_to($val, &mut s);
                assert_eq!(s, $base83);
                assert_eq!(encode($val), $base83);

                assert_eq!(decode($base83), Ok($val));
                assert_eq!(decode_ascii($base83), $val);
                assert_eq!(decode(&encode($val)), Ok($val));
            }
        };

        (($name:ident, $val:expr, $base83:expr); $(($name2:ident, $val2:expr, $base832:expr));+) => {
            gen_enc_dec_test!{ ($name, $val, $base83) }
            gen_enc_dec_test!{ $(($name2, $val2, $base832));+ }
        };
    }

    macro_rules! gen_enc_fixed_test {
        (($name:ident, $val:expr, $base83:expr)) => {
            #[test]
            fn $name() {
                let mut s = String::with_capacity(6);
                encode_fixed_to($val, $base83.len() as u8, &mut s);
                assert_eq!(s, $base83);
                assert_eq!(encode_fixed($val, $base83.len() as u8), $base83);
                assert_eq!(decode(&s), Ok($val));
                assert_eq!(decode_ascii(&s), $val);
            }
        };

        (($name:ident, $val:expr, $base83:expr); $(($name_s:ident, $val_s:expr, $base83_s:expr));+) => {
            gen_enc_fixed_test!{ ($name, $val, $base83) }
            gen_enc_fixed_test!{ $(($name_s, $val_s, $base83_s));+ }
        };
    }

    gen_enc_dec_test! {
        (test_enc_dec_zero, 0, "0");
        (test_enc_dec_one, 1, "1");
        (test_enc_dec_8bits, 42, "g");
        (test_enc_dec_16bits, 1234, "E=");
        (test_enc_dec_17bits, 65540, "9gr");
        (test_enc_dec_24bits, 0xcafeee, "NMAj");
        (test_enc_dec_32bits, 0xC0deCafe, "-FCDo");
        (test_enc_dec_max, u32::MAX, "17fd^]")
    }

    gen_enc_fixed_test! {
        (test_enc_fixed_zero, 0, "0000");
        (test_enc_fixed_one, 1, "001");
        (test_enc_fixed_8bits, 42, "g");
        (test_enc_fixed_16bits, 1234, "E=");
        (test_enc_fixed_17bits, 65540, "09gr");
        (test_enc_fixed_24bits, 0xcafeee, "0NMAj");
        (test_enc_fixed_32bits, 0xC0deCafe, "-FCDo");
        (test_enc_fixed_max, u32::MAX, "17fd^]")
    }

    #[test]
    fn decode_invalid() {
        assert_eq!(decode("BAD°"), Err(Base83ConversionError::InvalidChar));
    }

    #[test]
    fn decode_overflow() {
        assert_eq!(decode("18fd^]"), Err(Base83ConversionError::Overflow));
        assert_eq!(decode("17fd^^"), Err(Base83ConversionError::Overflow));
    }
}
