use uuid::Uuid;

use super::byte_utils::{little_endian_to_i32, little_endian_to_i64};

pub fn as_i32_le(bytes: &[u8]) -> i32 {
    little_endian_to_i32(bytes, 0)
}

pub fn as_i64_le(bytes: &[u8]) -> i64 {
    little_endian_to_i64(bytes, 0)
}

pub fn as_uuid(bytes: &[u8]) -> Uuid {
    Uuid::from_bytes(bytes[..16].try_into().expect("sixteen UUID bytes"))
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::{as_i32_le, as_i64_le, as_uuid};

    #[test]
    fn reads_i32_little_endian() {
        assert_eq!(as_i32_le(&[0x04, 0x03, 0x02, 0x01]), 0x01020304);
    }

    #[test]
    fn reads_i64_little_endian() {
        assert_eq!(
            as_i64_le(&[0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]),
            0x0102030405060708
        );
    }

    #[test]
    fn reads_uuid_with_java_byte_buffer_order() {
        let bytes = [
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc,
            0xde, 0xf0,
        ];

        assert_eq!(
            as_uuid(&bytes),
            Uuid::parse_str("12345678-9abc-def0-1234-56789abcdef0").unwrap()
        );
    }
}
