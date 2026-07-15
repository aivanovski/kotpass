use crate::constants::VariantTypeId;

/// Represents a variant data type item within a KDBX file header.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VariantItem {
    /// Unsigned 32-bit integer variant item.
    UInt32(u32),
    /// Unsigned 64-bit integer variant item.
    UInt64(u64),
    /// Boolean variant item.
    Bool(bool),
    /// Signed 32-bit integer variant item.
    Int32(i32),
    /// Signed 64-bit integer variant item.
    Int64(i64),
    /// UTF-8 encoded string variant item.
    StringUtf8(String),
    /// Byte array variant item.
    Bytes(Vec<u8>),
}

impl VariantItem {
    pub const fn type_id(&self) -> VariantTypeId {
        match self {
            Self::UInt32(_) => VariantTypeId::UInt32,
            Self::UInt64(_) => VariantTypeId::UInt64,
            Self::Bool(_) => VariantTypeId::Bool,
            Self::Int32(_) => VariantTypeId::Int32,
            Self::Int64(_) => VariantTypeId::Int64,
            Self::StringUtf8(_) => VariantTypeId::StringUtf8,
            Self::Bytes(_) => VariantTypeId::Bytes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::VariantItem;
    use crate::constants::VariantTypeId;

    #[test]
    fn item_type_ids_match_kotlin_variants() {
        let cases = [
            (VariantItem::UInt32(1), VariantTypeId::UInt32),
            (VariantItem::UInt64(1), VariantTypeId::UInt64),
            (VariantItem::Bool(true), VariantTypeId::Bool),
            (VariantItem::Int32(-1), VariantTypeId::Int32),
            (VariantItem::Int64(-1), VariantTypeId::Int64),
            (
                VariantItem::StringUtf8("value".to_owned()),
                VariantTypeId::StringUtf8,
            ),
            (VariantItem::Bytes(vec![1, 2, 3]), VariantTypeId::Bytes),
        ];

        for (item, expected_type_id) in cases {
            assert_eq!(item.type_id(), expected_type_id);
        }
    }

    #[test]
    fn items_compare_by_variant_and_value() {
        assert_eq!(VariantItem::UInt32(42), VariantItem::UInt32(42));
        assert_ne!(VariantItem::UInt32(42), VariantItem::Int32(42));
        assert_ne!(
            VariantItem::StringUtf8("42".to_owned()),
            VariantItem::Bytes(b"42".to_vec())
        );
    }
}
