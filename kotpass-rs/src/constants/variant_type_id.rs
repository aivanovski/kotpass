#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum VariantTypeId {
    None = 0x00,
    UInt32 = 0x04,
    UInt64 = 0x05,
    Bool = 0x08,
    Int32 = 0x0C,
    Int64 = 0x0D,
    StringUtf8 = 0x18,
    Bytes = 0x42,
}

impl VariantTypeId {
    pub const fn id(self) -> u8 {
        self as u8
    }

    pub const fn from_id(id: u8) -> Option<Self> {
        match id {
            0x00 => Some(Self::None),
            0x04 => Some(Self::UInt32),
            0x05 => Some(Self::UInt64),
            0x08 => Some(Self::Bool),
            0x0C => Some(Self::Int32),
            0x0D => Some(Self::Int64),
            0x18 => Some(Self::StringUtf8),
            0x42 => Some(Self::Bytes),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::VariantTypeId;

    #[test]
    fn ids_match_kotlin_constants() {
        assert_eq!(VariantTypeId::None.id(), 0);
        assert_eq!(VariantTypeId::UInt32.id(), 0x04);
        assert_eq!(VariantTypeId::UInt64.id(), 0x05);
        assert_eq!(VariantTypeId::Bool.id(), 0x08);
        assert_eq!(VariantTypeId::Int32.id(), 0x0C);
        assert_eq!(VariantTypeId::Int64.id(), 0x0D);
        assert_eq!(VariantTypeId::StringUtf8.id(), 0x18);
        assert_eq!(VariantTypeId::Bytes.id(), 0x42);
    }

    #[test]
    fn parses_known_ids() {
        assert_eq!(VariantTypeId::from_id(0x00), Some(VariantTypeId::None));
        assert_eq!(
            VariantTypeId::from_id(0x18),
            Some(VariantTypeId::StringUtf8)
        );
        assert_eq!(VariantTypeId::from_id(0x42), Some(VariantTypeId::Bytes));
        assert_eq!(VariantTypeId::from_id(0x43), None);
    }
}
