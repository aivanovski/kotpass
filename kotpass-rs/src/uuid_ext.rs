use uuid::Uuid;

use crate::io::base16::encode_hex;

pub fn to_hex_string(uuid: Uuid) -> String {
    encode_hex(uuid.as_bytes())
}

pub fn is_zero(uuid: Uuid) -> bool {
    uuid.as_u128() == 0
}

pub fn is_null_or_zero(uuid: Option<Uuid>) -> bool {
    uuid.is_none_or(is_zero)
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::{is_null_or_zero, is_zero, to_hex_string};

    #[test]
    fn converts_uuid_to_java_byte_buffer_hex_order() {
        let uuid = Uuid::parse_str("12345678-9abc-def0-1234-56789abcdef0").unwrap();

        assert_eq!(to_hex_string(uuid), "123456789abcdef0123456789abcdef0");
    }

    #[test]
    fn detects_zero_uuid() {
        let zero = Uuid::from_u128(0);
        let non_zero = Uuid::parse_str("12345678-9abc-def0-1234-56789abcdef0").unwrap();

        assert!(is_zero(zero));
        assert!(!is_zero(non_zero));
    }

    #[test]
    fn detects_null_or_zero_uuid() {
        let zero = Uuid::from_u128(0);
        let non_zero = Uuid::parse_str("12345678-9abc-def0-1234-56789abcdef0").unwrap();

        assert!(is_null_or_zero(None));
        assert!(is_null_or_zero(Some(zero)));
        assert!(!is_null_or_zero(Some(non_zero)));
    }
}
