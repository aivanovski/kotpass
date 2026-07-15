use thiserror::Error;

const HEX_MAP: &[u8; 16] = b"0123456789abcdef";

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum HexError {
    #[error("Invalid hex string length.")]
    InvalidLength,

    #[error("Unexpected hex char: {0}")]
    UnexpectedChar(char),
}

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push(char::from(HEX_MAP[usize::from(byte >> 4)]));
        result.push(char::from(HEX_MAP[usize::from(byte & 0x0f)]));
    }
    result
}

pub fn decode_hex_to_array(hex: &str) -> Result<Vec<u8>, HexError> {
    if hex.len() % 2 != 0 {
        return Err(HexError::InvalidLength);
    }

    let mut result = Vec::with_capacity(hex.len() / 2);
    let mut chars = hex.chars();
    while let Some(high) = chars.next() {
        let low = chars.next().expect("hex string length checked");
        result.push((decode_hex_digit(high)? << 4) + decode_hex_digit(low)?);
    }

    Ok(result)
}

fn decode_hex_digit(c: char) -> Result<u8, HexError> {
    match c {
        '0'..='9' => Ok(c as u8 - b'0'),
        'a'..='f' => Ok(c as u8 - b'a' + 10),
        'A'..='F' => Ok(c as u8 - b'A' + 10),
        _ => Err(HexError::UnexpectedChar(c)),
    }
}

#[cfg(test)]
mod tests {
    use super::{HexError, decode_hex_to_array, encode_hex};

    #[test]
    fn encodes_lowercase_hex() {
        assert_eq!(encode_hex(&[0x00, 0x0f, 0x10, 0xab, 0xff]), "000f10abff");
    }

    #[test]
    fn decodes_lowercase_hex() {
        assert_eq!(
            decode_hex_to_array("000f10abff").unwrap(),
            vec![0x00, 0x0f, 0x10, 0xab, 0xff]
        );
    }

    #[test]
    fn decodes_uppercase_hex() {
        assert_eq!(
            decode_hex_to_array("DEADBEEF").unwrap(),
            vec![0xde, 0xad, 0xbe, 0xef]
        );
    }

    #[test]
    fn rejects_odd_length() {
        assert_eq!(decode_hex_to_array("abc"), Err(HexError::InvalidLength));
    }

    #[test]
    fn rejects_unexpected_characters() {
        assert_eq!(
            decode_hex_to_array("0x"),
            Err(HexError::UnexpectedChar('x'))
        );
    }
}
