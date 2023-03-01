const CHARACTERS: [char; 83] = [
    '0','1','2','3','4','5','6','7','8','9','A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z','a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z','#','$','%','*','+',',','-','.',':',';','=','?','@','[',']','^','_','{','|','}','~',
];

pub fn encode(mut n: u32) -> String {
    if n == 0 {
        return CHARACTERS[0].to_string();
    }

    let mut stack: [char; 6] = ['\0'; 6];
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
        str.push(stack[i]);
    }
    str
}

pub fn encode_to(mut n: u32, str: &mut String) {
    if n == 0 {
        str.push(CHARACTERS[0]);
        return;
    }

    let mut stack: [char; 6] = ['\0'; 6];
    let mut i = 0;

    while n > 0 {
        stack[i] = CHARACTERS[(n % 83) as usize];
        n /= 83;
        i += 1;
    }

    while i > 0 { // append to string in the reverse order
        i -= 1;
        str.push(stack[i]);
    }
}

pub fn encode_fixed(mut n: u32, iters: u8) -> String {
    debug_assert!(iters <= 6);

    let mut stack: [char; 6] = ['\0'; 6];
    let mut i = 0;

    for _ in 0..iters {
        stack[i] = CHARACTERS[(n % 83) as usize];
        n /= 83;
        i += 1;
    }

    // allocate string
    let mut str = String::with_capacity(i);
    while i > 0 { // append to string in the reverse order
        i -= 1;
        str.push(stack[i]);
    }
    str
}

pub fn encode_fixed_to(mut n: u32, iters: u8, str: &mut String) {
    debug_assert!(iters <= 6);

    let mut stack: [char; 6] = ['\0'; 6];
    let mut i = 0;

    for _ in 0..iters {
        stack[i] = CHARACTERS[(n % 83) as usize];
        n /= 83;
        i += 1;
    }

    while i > 0 { // append to string in the reverse order
        i -= 1;
        str.push(stack[i]);
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

pub fn decode_ascii(s: &str) -> u32 {
    debug_assert!(s.is_ascii());

    s.chars()
        .map(|c| DIGITS[c as usize] as u32)
        .fold(0, |acc, c| acc * 83 + c)
}

pub fn decode(s: &str) -> Option<u32> {
    let mut n: u32 = 0;

    let mut chars = s.chars();
    for _ in 0..5 { // no overflow until 6th character
        match chars.next() {
            Some(c) if c.is_ascii() => {
                n = n * 83 + DIGITS[c as usize] as u32;
            },
            Some(_) => return None, // invalid char
            None => return Some(n) // end of string
        }
    }

    match chars.next() {
        Some(c) if c.is_ascii() => {
            // overflow check
            n.checked_mul(83u32)?.checked_add(DIGITS[c as usize] as u32)
        },
        Some(_) => None, // invalid char
        None => Some(n) // end of string
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

                assert_eq!(decode($base83), Some($val));
                assert_eq!(decode_ascii($base83), $val);
                assert_eq!(decode(&encode($val)), Some($val));
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
                assert_eq!(decode(&s), Some($val));
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
        assert_eq!(decode("BAD°"), None);
    }

    #[test]
    fn decode_overflow() {
        assert_eq!(decode("18fd^]"), None);
    }
}
