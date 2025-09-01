#[derive(Debug, Clone, Copy)]
pub struct EmbeddedStr {
    bytes: [u8; 3],
    len: u8,
}

impl EmbeddedStr {
    pub const fn new(symbol: char) -> Self {
        let mut temp_bytes = [0; 4];
        let encoded = symbol.encode_utf8(&mut temp_bytes);
        let len = encoded.len() as u8;

        if len <= 3 {
            let mut bytes = [0; 3];
            let mut i = 0;
            while i < len as usize {
                bytes[i] = temp_bytes[i];
                i += 1;
            }
            Self { bytes, len }
        } else {
            Self {
                bytes: [b' ', 0, 0],
                len: 1,
            }
        }
    }

    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl From<char> for EmbeddedStr {
    fn from(c: char) -> Self {
        let c = c as u32;

        // Fast path for ASCII (0-127)
        if c < 128 {
            return Self {
                bytes: [c as u8, 0, 0],
                len: 1,
            };
        }

        // Direct UTF-8 encoding without temp array
        let mut bytes = [0u8; 3];
        let len = if c < 0x800 {
            // 2-byte UTF-8: 110xxxxx 10xxxxxx
            bytes[0] = 0xC0 | ((c >> 6) as u8);
            bytes[1] = 0x80 | ((c & 0x3F) as u8);
            2
        } else if c < 0x10000 {
            // 3-byte UTF-8: 1110xxxx 10xxxxxx 10xxxxxx
            bytes[0] = 0xE0 | ((c >> 12) as u8);
            bytes[1] = 0x80 | (((c >> 6) & 0x3F) as u8);
            bytes[2] = 0x80 | ((c & 0x3F) as u8);
            3
        } else {
            // 4-byte chars fallback to space
            bytes[0] = b' ';
            1
        };

        Self { bytes, len }
    }
}

impl From<&str> for EmbeddedStr {
    fn from(s: &str) -> Self {
        let bytes = s.as_bytes();

        // Fast paths
        if bytes.is_empty() {
            return Self {
                bytes: [b' ', 0, 0],
                len: 1,
            };
        }

        if bytes.len() <= 3 { // s.is_char_boundary(bytes.len()) {
            let mut result_bytes = [0u8; 3];
            result_bytes[..bytes.len()].copy_from_slice(bytes);
            Self {
                bytes: result_bytes,
                len: bytes.len() as u8,
            }
        } else {
            Self {
                bytes: [b' ', 0, 0],
                len: 1,
            }
        }
    }
}

impl AsRef<str> for EmbeddedStr {
    fn as_ref(&self) -> &str {
        #[allow(unsafe_code)]
        unsafe { core::str::from_utf8_unchecked(&self.bytes[..self.len as usize]) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_char_ascii() {
        let embedded = EmbeddedStr::from('A');
        assert_eq!(embedded.as_str(), "A");
        assert_eq!(embedded.len, 1);
        assert_eq!(embedded.bytes[0], b'A');
    }

    #[test]
    fn test_from_char_two_byte() {
        let embedded = EmbeddedStr::from('é');
        assert_eq!(embedded.as_str(), "é");
        assert_eq!(embedded.len, 2);
        // 'é' is encoded as [0xC3, 0xA9]
        assert_eq!(embedded.bytes[0], 0xC3);
        assert_eq!(embedded.bytes[1], 0xA9);
    }

    #[test]
    fn test_from_char_three_byte() {
        let embedded = EmbeddedStr::from('€');
        assert_eq!(embedded.as_str(), "€");
        assert_eq!(embedded.len, 3);
        // '€' is encoded as [0xE2, 0x82, 0xAC]
        assert_eq!(embedded.bytes[0], 0xE2);
        assert_eq!(embedded.bytes[1], 0x82);
        assert_eq!(embedded.bytes[2], 0xAC);
    }

    #[test]
    fn test_from_char_four_byte_fallback() {
        let embedded = EmbeddedStr::from('🚀');
        assert_eq!(embedded.as_str(), " ");
        assert_eq!(embedded.len, 1);
        assert_eq!(embedded.bytes[0], b' ');
    }

    #[test]
    fn test_from_str_empty() {
        let embedded = EmbeddedStr::from("");
        assert_eq!(embedded.as_str(), " ");
        assert_eq!(embedded.len, 1);
    }

    #[test]
    fn test_from_str_single_ascii() {
        let embedded = EmbeddedStr::from("A");
        assert_eq!(embedded.as_str(), "A");
        assert_eq!(embedded.len, 1);
    }

    #[test]
    fn test_from_str_two_byte() {
        let embedded = EmbeddedStr::from("é");
        assert_eq!(embedded.as_str(), "é");
        assert_eq!(embedded.len, 2);
    }

    #[test]
    fn test_from_str_three_byte() {
        let embedded = EmbeddedStr::from("€");
        assert_eq!(embedded.as_str(), "€");
        assert_eq!(embedded.len, 3);
    }

    #[test]
    fn test_from_str_multiple_chars() {
        // Should only take first character
        let embedded = EmbeddedStr::from("hello");
        assert_eq!(embedded.as_str(), "h");
        assert_eq!(embedded.len, 1);
    }

    #[test]
    fn test_from_str_four_byte_emoji() {
        // Should fallback to space for 4-byte chars
        let embedded = EmbeddedStr::from("🚀");
        assert_eq!(embedded.as_str(), " ");
        assert_eq!(embedded.len, 1);
    }

    #[test]
    fn test_as_ref() {
        let embedded = EmbeddedStr::from('π');
        assert_eq!(embedded.as_ref(), "π");
        assert_eq!(embedded.as_str(), "π");
    }

    #[test]
    fn test_various_unicode() {
        let test_cases = [
            ('A', "A", 1),      // ASCII
            ('ñ', "ñ", 2),      // Latin-1 Supplement
            ('π', "π", 2),      // Greek
            ('中', "中", 3),     // CJK
            ('™', "™", 3),      // Symbol
            ('\n', "\n", 1),    // Control character
        ];

        for (input_char, expected_str, expected_len) in test_cases {
            let embedded = EmbeddedStr::from(input_char);
            assert_eq!(
                embedded.as_str(),
                expected_str,
                "Failed for char: {}",
                input_char
            );
            assert_eq!(
                embedded.len, expected_len,
                "Wrong length for char: {}",
                input_char
            );
        }
    }

    #[test]
    fn test_clone_copy() {
        let original = EmbeddedStr::from('€');
        let copied = original;
        let cloned = original.clone();

        assert_eq!(original.as_str(), "€");
        assert_eq!(copied.as_str(), "€");
        assert_eq!(cloned.as_str(), "€");
    }

    #[test]
    fn test_const_new() {
        const EMBEDDED: EmbeddedStr = EmbeddedStr::new('A');
        assert_eq!(EMBEDDED.as_str(), "A");
    }
}