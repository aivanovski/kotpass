use indexmap::IndexMap;

use crate::constants::VariantTypeId;
use crate::error::FormatError;
use crate::io::buffered_stream::BufferedStream;
use crate::io::real_buffered_stream::RealBufferedStream;

use super::VariantItem;

pub type VariantItems = IndexMap<String, VariantItem>;

/// Reads and writes KDBX variant dictionaries used by header fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VariantDictionary;

impl VariantDictionary {
    const VERSION: u16 = 0x0100;
    const VERSION_FILTER: u16 = 0xFF00;

    pub fn read_from(data: &[u8]) -> Result<VariantItems, FormatError> {
        let mut result = IndexMap::new();
        let mut buffer = RealBufferedStream::from_bytes(data.to_vec());
        let version = read_u16_le(&mut buffer)?;

        if (version & Self::VERSION_FILTER) > (Self::VERSION & Self::VERSION_FILTER) {
            return Err(FormatError::InvalidHeader(
                "Variant dictionary version exceeds expected value.".to_owned(),
            ));
        }

        loop {
            let item_type = read_type_id(&mut buffer)?;

            if item_type == VariantTypeId::None {
                break;
            }

            let key_length = read_length(&mut buffer, LengthKind::Key)?;
            if key_length == 0 {
                return Err(FormatError::InvalidHeader(
                    "Variant dictionary item's key has zero length.".to_owned(),
                ));
            }

            let key = read_string(&mut buffer, key_length)?;
            let value_length = read_length(&mut buffer, LengthKind::Value)?;
            let item = read_item(&mut buffer, item_type, value_length)?;

            result.insert(key, item);
        }

        Ok(result)
    }

    pub fn write_to_bytes(items: &VariantItems) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&Self::VERSION.to_le_bytes());

        for (key, item) in items {
            bytes.push(item.type_id().id());
            write_len(&mut bytes, key.len());
            bytes.extend_from_slice(key.as_bytes());
            write_item(&mut bytes, item);
        }

        bytes.push(VariantTypeId::None.id());
        bytes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LengthKind {
    Key,
    Value,
}

fn read_type_id<S: BufferedStream>(buffer: &mut S) -> Result<VariantTypeId, FormatError> {
    let id = read_u8(buffer)?;
    VariantTypeId::from_id(id).ok_or_else(|| {
        FormatError::InvalidHeader(format!("Unknown variant dictionary item type: {id}."))
    })
}

fn read_item<S: BufferedStream>(
    buffer: &mut S,
    item_type: VariantTypeId,
    value_length: usize,
) -> Result<VariantItem, FormatError> {
    match item_type {
        VariantTypeId::None => Err(FormatError::InvalidHeader(
            "Unexpected variant dictionary terminator.".to_owned(),
        )),
        VariantTypeId::UInt32 => {
            require_value_length(item_type, value_length, size_of::<u32>())?;
            Ok(VariantItem::UInt32(read_u32_le(buffer)?))
        }
        VariantTypeId::UInt64 => {
            require_value_length(item_type, value_length, size_of::<u64>())?;
            Ok(VariantItem::UInt64(read_u64_le(buffer)?))
        }
        VariantTypeId::Bool => {
            require_value_length(item_type, value_length, size_of::<u8>())?;
            Ok(VariantItem::Bool(read_u8(buffer)? != 0))
        }
        VariantTypeId::Int32 => {
            require_value_length(item_type, value_length, size_of::<i32>())?;
            Ok(VariantItem::Int32(read_i32_le(buffer)?))
        }
        VariantTypeId::Int64 => {
            require_value_length(item_type, value_length, size_of::<i64>())?;
            Ok(VariantItem::Int64(read_i64_le(buffer)?))
        }
        VariantTypeId::StringUtf8 => {
            Ok(VariantItem::StringUtf8(read_string(buffer, value_length)?))
        }
        VariantTypeId::Bytes => Ok(VariantItem::Bytes(read_bytes(buffer, value_length)?)),
    }
}

fn require_value_length(
    item_type: VariantTypeId,
    actual: usize,
    expected: usize,
) -> Result<(), FormatError> {
    if actual == expected {
        Ok(())
    } else {
        Err(FormatError::InvalidHeader(format!(
            "Invalid item's value length for type: {}.",
            item_type_name(item_type)
        )))
    }
}

fn item_type_name(item_type: VariantTypeId) -> &'static str {
    match item_type {
        VariantTypeId::None => "None",
        VariantTypeId::UInt32 => "UInt32",
        VariantTypeId::UInt64 => "UInt64",
        VariantTypeId::Bool => "Bool",
        VariantTypeId::Int32 => "Int32",
        VariantTypeId::Int64 => "Int64",
        VariantTypeId::StringUtf8 => "StringUtf8",
        VariantTypeId::Bytes => "Bytes",
    }
}

fn read_length<S: BufferedStream>(buffer: &mut S, kind: LengthKind) -> Result<usize, FormatError> {
    let length = read_i32_le(buffer)?;
    match (kind, length) {
        (LengthKind::Key, ..=0) => Ok(0),
        (LengthKind::Value, ..=-1) => Err(FormatError::InvalidHeader(
            "Variant dictionary item's value has invalid length.".to_owned(),
        )),
        _ => Ok(length as usize),
    }
}

fn read_string<S: BufferedStream>(buffer: &mut S, length: usize) -> Result<String, FormatError> {
    String::from_utf8(read_bytes(buffer, length)?).map_err(|error| {
        FormatError::InvalidHeader(format!("Variant dictionary string is not UTF-8: {error}."))
    })
}

fn read_bytes<S: BufferedStream>(buffer: &mut S, length: usize) -> Result<Vec<u8>, FormatError> {
    buffer
        .read_byte_string_len(length)
        .map_err(|error| FormatError::InvalidHeader(error.to_string()))
}

fn read_u8<S: BufferedStream>(buffer: &mut S) -> Result<u8, FormatError> {
    buffer
        .read_byte()
        .map_err(|error| FormatError::InvalidHeader(error.to_string()))
}

fn read_i32_le<S: BufferedStream>(buffer: &mut S) -> Result<i32, FormatError> {
    buffer
        .read_int_le()
        .map_err(|error| FormatError::InvalidHeader(error.to_string()))
}

fn read_i64_le<S: BufferedStream>(buffer: &mut S) -> Result<i64, FormatError> {
    buffer
        .read_long_le()
        .map_err(|error| FormatError::InvalidHeader(error.to_string()))
}

fn read_u16_le<S: BufferedStream>(buffer: &mut S) -> Result<u16, FormatError> {
    let bytes = buffer
        .read_array::<2>()
        .map_err(|error| FormatError::InvalidHeader(error.to_string()))?;
    Ok(u16::from_le_bytes(bytes))
}

fn read_u32_le<S: BufferedStream>(buffer: &mut S) -> Result<u32, FormatError> {
    let bytes = buffer
        .read_array::<4>()
        .map_err(|error| FormatError::InvalidHeader(error.to_string()))?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_u64_le<S: BufferedStream>(buffer: &mut S) -> Result<u64, FormatError> {
    let bytes = buffer
        .read_array::<8>()
        .map_err(|error| FormatError::InvalidHeader(error.to_string()))?;
    Ok(u64::from_le_bytes(bytes))
}

fn write_item(bytes: &mut Vec<u8>, item: &VariantItem) {
    match item {
        VariantItem::UInt32(value) => {
            write_len(bytes, size_of::<u32>());
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        VariantItem::UInt64(value) => {
            write_len(bytes, size_of::<u64>());
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        VariantItem::Bool(value) => {
            write_len(bytes, size_of::<u8>());
            bytes.push(u8::from(*value));
        }
        VariantItem::Int32(value) => {
            write_len(bytes, size_of::<i32>());
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        VariantItem::Int64(value) => {
            write_len(bytes, size_of::<i64>());
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        VariantItem::StringUtf8(value) => {
            write_len(bytes, value.len());
            bytes.extend_from_slice(value.as_bytes());
        }
        VariantItem::Bytes(value) => {
            write_len(bytes, value.len());
            bytes.extend_from_slice(value);
        }
    }
}

fn write_len(bytes: &mut Vec<u8>, length: usize) {
    let length = i32::try_from(length).expect("variant dictionary item length exceeds i32");
    bytes.extend_from_slice(&length.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use super::{VariantDictionary, VariantItem};
    use crate::constants::VariantTypeId;
    use crate::io::base16::decode_hex_to_array;

    #[test]
    fn reads_and_writes_all_item_types() {
        let mut items = IndexMap::new();
        items.insert("u32".to_owned(), VariantItem::UInt32(u32::MAX));
        items.insert("u64".to_owned(), VariantItem::UInt64(u64::MAX));
        items.insert("bool".to_owned(), VariantItem::Bool(true));
        items.insert("i32".to_owned(), VariantItem::Int32(-42));
        items.insert("i64".to_owned(), VariantItem::Int64(-43));
        items.insert(
            "str".to_owned(),
            VariantItem::StringUtf8("hello".to_owned()),
        );
        items.insert("bytes".to_owned(), VariantItem::Bytes(vec![0, 1, 2, 3]));

        let bytes = VariantDictionary::write_to_bytes(&items);
        let decoded = VariantDictionary::read_from(&bytes).unwrap();

        assert_eq!(decoded, items);
        assert_eq!(VariantDictionary::write_to_bytes(&decoded), bytes);
    }

    #[test]
    fn reads_kotlin_kdf_params_fixture() {
        let bytes = include_bytes!("../../../../kotpass/kotpass/src/test/resources/kdf_params");
        let decoded = VariantDictionary::read_from(bytes).unwrap();

        assert_eq!(
            decoded.get("$UUID"),
            Some(&VariantItem::Bytes(
                decode_hex_to_array("ef636ddf8c29444b91f7a9a403e30a0c").unwrap()
            ))
        );
        assert_eq!(
            VariantDictionary::read_from(&VariantDictionary::write_to_bytes(&decoded)).unwrap(),
            decoded
        );
    }

    #[test]
    fn rejects_major_versions_above_supported_version() {
        let error = VariantDictionary::read_from(&[0x00, 0x02, VariantTypeId::None.id()])
            .expect_err("version should be rejected");

        assert_eq!(
            error.to_string(),
            "Variant dictionary version exceeds expected value."
        );
    }

    #[test]
    fn rejects_zero_length_keys() {
        let bytes = [
            0x00,
            0x01,
            VariantTypeId::Bool.id(),
            0,
            0,
            0,
            0,
            1,
            0,
            0,
            0,
            1,
            VariantTypeId::None.id(),
        ];

        let error = VariantDictionary::read_from(&bytes).expect_err("key should be rejected");

        assert_eq!(
            error.to_string(),
            "Variant dictionary item's key has zero length."
        );
    }

    #[test]
    fn rejects_negative_value_lengths() {
        let bytes = [
            0x00,
            0x01,
            VariantTypeId::Bytes.id(),
            1,
            0,
            0,
            0,
            b'k',
            0xff,
            0xff,
            0xff,
            0xff,
            VariantTypeId::None.id(),
        ];

        let error = VariantDictionary::read_from(&bytes).expect_err("length should be rejected");

        assert_eq!(
            error.to_string(),
            "Variant dictionary item's value has invalid length."
        );
    }

    #[test]
    fn rejects_invalid_fixed_value_lengths() {
        let bytes = [
            0x00,
            0x01,
            VariantTypeId::UInt32.id(),
            1,
            0,
            0,
            0,
            b'k',
            3,
            0,
            0,
            0,
            1,
            2,
            3,
            VariantTypeId::None.id(),
        ];

        let error = VariantDictionary::read_from(&bytes).expect_err("length should be rejected");

        assert_eq!(
            error.to_string(),
            "Invalid item's value length for type: UInt32."
        );
    }

    #[test]
    fn rejects_unknown_item_types() {
        let error = VariantDictionary::read_from(&[0x00, 0x01, 0xff])
            .expect_err("unknown type should be rejected");

        assert_eq!(
            error.to_string(),
            "Unknown variant dictionary item type: 255."
        );
    }
}
