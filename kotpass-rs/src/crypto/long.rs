use super::byte_utils::{i64_to_little_endian, little_endian_to_i64};

pub fn to_byte_array(value: i64) -> [u8; 8] {
    i64_to_little_endian(value)
}

pub fn from_byte_array(bytes: &[u8]) -> i64 {
    little_endian_to_i64(bytes, 0)
}

#[cfg(test)]
mod tests {
    use super::{from_byte_array, to_byte_array};

    #[test]
    fn converts_long_to_little_endian_bytes() {
        assert_eq!(
            to_byte_array(0x0102030405060708),
            [0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]
        );
    }

    #[test]
    fn converts_little_endian_bytes_to_long() {
        assert_eq!(
            from_byte_array(&[0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]),
            0x0102030405060708
        );
    }

    #[test]
    fn preserves_negative_values() {
        assert_eq!(to_byte_array(-1), [0xff; 8]);
        assert_eq!(from_byte_array(&[0xff; 8]), -1);
    }
}
