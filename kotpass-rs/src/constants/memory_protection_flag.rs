use super::BasicField;

/// Specifies which sensitive fields of an entry are protected in memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryProtectionFlag {
    Title,
    UserName,
    Password,
    Url,
    Notes,
}

impl MemoryProtectionFlag {
    pub const ALL: [Self; 5] = [
        Self::Title,
        Self::UserName,
        Self::Password,
        Self::Url,
        Self::Notes,
    ];

    pub const fn value(self) -> &'static str {
        match self {
            Self::Title => "ProtectTitle",
            Self::UserName => "ProtectUserName",
            Self::Password => "ProtectPassword",
            Self::Url => "ProtectURL",
            Self::Notes => "ProtectNotes",
        }
    }

    pub const fn to_basic_field(self) -> BasicField {
        match self {
            Self::Title => BasicField::Title,
            Self::UserName => BasicField::UserName,
            Self::Password => BasicField::Password,
            Self::Url => BasicField::Url,
            Self::Notes => BasicField::Notes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BasicField, MemoryProtectionFlag};

    #[test]
    fn values_match_format_xml_memory_protection_tags() {
        assert_eq!(MemoryProtectionFlag::Title.value(), "ProtectTitle");
        assert_eq!(MemoryProtectionFlag::UserName.value(), "ProtectUserName");
        assert_eq!(MemoryProtectionFlag::Password.value(), "ProtectPassword");
        assert_eq!(MemoryProtectionFlag::Url.value(), "ProtectURL");
        assert_eq!(MemoryProtectionFlag::Notes.value(), "ProtectNotes");
    }

    #[test]
    fn maps_to_basic_fields() {
        assert_eq!(
            MemoryProtectionFlag::Title.to_basic_field(),
            BasicField::Title
        );
        assert_eq!(
            MemoryProtectionFlag::UserName.to_basic_field(),
            BasicField::UserName
        );
        assert_eq!(
            MemoryProtectionFlag::Password.to_basic_field(),
            BasicField::Password
        );
        assert_eq!(MemoryProtectionFlag::Url.to_basic_field(), BasicField::Url);
        assert_eq!(
            MemoryProtectionFlag::Notes.to_basic_field(),
            BasicField::Notes
        );
    }
}
