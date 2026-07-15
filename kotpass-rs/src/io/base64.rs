use ::base64::{Engine, engine::general_purpose};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Base64Error {
    #[error("Unexpected character at: {0}.")]
    UnexpectedCharacter(usize),

    #[error("Invalid last char.")]
    InvalidLastChar,
}

pub fn encode_base64(bytes: &[u8]) -> String {
    general_purpose::STANDARD.encode(bytes)
}

pub fn encode_base64_url_safe(bytes: &[u8]) -> String {
    general_purpose::URL_SAFE.encode(bytes)
}

pub fn decode_base64_to_array(input: &str) -> Result<Vec<u8>, Base64Error> {
    let mut limit = input.len();
    while limit > 0 {
        let byte = input.as_bytes()[limit - 1];
        if !matches!(byte, b'=' | b'\n' | b'\r' | b' ' | b'\t') {
            break;
        }
        limit -= 1;
    }

    let mut out = Vec::with_capacity(limit * 6 / 8);
    let mut in_count = 0usize;
    let mut word = 0u32;

    for (pos, c) in input[..limit].chars().enumerate() {
        let bits = match c {
            'A'..='Z' => c as u32 - 'A' as u32,
            'a'..='z' => c as u32 - 'a' as u32 + 26,
            '0'..='9' => c as u32 - '0' as u32 + 52,
            '+' | '-' => 62,
            '/' | '_' => 63,
            '\n' | '\r' | ' ' | '\t' | '=' => continue,
            _ => return Err(Base64Error::UnexpectedCharacter(pos)),
        };

        word = (word << 6) | bits;
        in_count += 1;

        if in_count % 4 == 0 {
            out.push((word >> 16) as u8);
            out.push((word >> 8) as u8);
            out.push(word as u8);
            word = 0;
        }
    }

    match in_count % 4 {
        0 => {}
        1 => return Err(Base64Error::InvalidLastChar),
        2 => {
            word <<= 12;
            out.push((word >> 16) as u8);
        }
        3 => {
            word <<= 6;
            out.push((word >> 16) as u8);
            out.push((word >> 8) as u8);
        }
        _ => unreachable!(),
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{Base64Error, decode_base64_to_array, encode_base64, encode_base64_url_safe};

    #[test]
    fn encodes_standard_base64_with_padding() {
        assert_eq!(encode_base64(b"hello"), "aGVsbG8=");
        assert_eq!(encode_base64(b""), "");
    }

    #[test]
    fn encodes_url_safe_base64_with_padding() {
        assert_eq!(encode_base64_url_safe(&[0xfb, 0xff]), "-_8=");
    }

    #[test]
    fn decodes_standard_base64() {
        assert_eq!(decode_base64_to_array("aGVsbG8=").unwrap(), b"hello");
    }

    #[test]
    fn decodes_url_safe_base64() {
        assert_eq!(decode_base64_to_array("-_8=").unwrap(), vec![0xfb, 0xff]);
    }

    #[test]
    fn decodes_without_padding() {
        assert_eq!(decode_base64_to_array("aGVsbG8").unwrap(), b"hello");
    }

    #[test]
    fn ignores_whitespace_and_padding() {
        assert_eq!(decode_base64_to_array(" aG Vs\nbG8=\t").unwrap(), b"hello");
    }

    #[test]
    fn rejects_single_trailing_base64_digit() {
        assert_eq!(
            decode_base64_to_array("A"),
            Err(Base64Error::InvalidLastChar)
        );
    }

    #[test]
    fn rejects_unexpected_characters() {
        assert_eq!(
            decode_base64_to_array("a?=="),
            Err(Base64Error::UnexpectedCharacter(1))
        );
    }
}
