const CHARACTERS: [char; 83] = [
    '0','1','2','3','4','5','6','7','8','9','A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z','a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z','#','$','%','*','+',',','-','.',':',';','=','?','@','[',']','^','_','{','|','}','~',
];

pub fn encode(mut n: u32) -> String {
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
    s.chars()
        .map(|c| DIGITS[c as usize] as u32)
        .fold(0, |acc, c| acc * 83 + c)
}

pub fn decode(s: &str) -> Option<u32> {
    let mut n = 0;

    for c in s.chars() {
        if c.is_ascii() {
            n = n * 83 + DIGITS[c as usize] as u32;
        } else {
            return None;
        }
    }

    Some(n)
}

#[cfg(test)]
mod tests {
    use super::{decode, decode_ascii, encode, encode_to};

    #[test]
    fn encode_8bits() {
        assert_eq!(encode(42), "g");
    }

    #[test]
    fn encode_16bits() {
        assert_eq!(encode(1234), "E=");
    }

    #[test]
    fn encode_24bits() {
        assert_eq!(encode(0xcafeee), "NMAj");
    }

    #[test]
    fn encode_32bits() {
        assert_eq!(encode(0xC0deCafe), "-FCDo");
    }

    #[test]
    fn encode_max() {
        assert_eq!(encode(u32::MAX), "17fd^]");
    }

    #[test]
    fn encode_to_8bits() {
        let mut s = String::with_capacity(6);
        encode_to(42, &mut s);
        assert_eq!(s, "g");
    }

    #[test]
    fn encode_to_16bits() {
        let mut s = String::with_capacity(6);
        encode_to(1234, &mut s);
        assert_eq!(s, "E=");
    }

    #[test]
    fn encode_to_24bits() {
        let mut s = String::with_capacity(6);
        encode_to(0xcafeee, &mut s);
        assert_eq!(s, "NMAj");
    }

    #[test]
    fn encode_to_32bits() {
        let mut s = String::with_capacity(6);
        encode_to(0xC0deCafe, &mut s);
        assert_eq!(s, "-FCDo");
    }

    #[test]
    fn encode_to_max() {
        let mut s = String::with_capacity(6);
        encode_to(u32::MAX, &mut s);
        assert_eq!(s, "17fd^]");
    }

    #[test]
    fn decode_ascii_8bits() {
        assert_eq!(decode_ascii("g"), 42);
    }

    #[test]
    fn decode_ascii_16bits() {
        assert_eq!(decode_ascii("E="), 1234);
    }

    #[test]
    fn decode_ascii_24bits() {
        assert_eq!(decode_ascii("NMAj"), 0xcafeee);
    }

    #[test]
    fn decode_ascii_32bits() {
        assert_eq!(decode_ascii("-FCDo"), 0xC0deCafe);
    }

    #[test]
    fn decode_ascii_max() {
        assert_eq!(decode_ascii("17fd^]"), u32::MAX);
    }

    #[test]
    fn decode_invalid() {
        assert_eq!(decode("BAD°"), None);
    }

    #[test]
    fn decode_max() {
        assert_eq!(decode("17fd^]"), Some(u32::MAX));
    }
}
