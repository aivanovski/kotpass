/// Represents dynamic placeholders used in KeePass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Placeholder {
    /// The entry's title: `{TITLE}`.
    Title,

    /// The entry's user name: `{USERNAME}`.
    UserName,

    /// The entry's password: `{PASSWORD}`.
    Password,

    /// The entry's URL: `{URL}`.
    Url,

    /// The entry's notes: `{NOTES}`.
    Notes,

    /// The entry's unique ID: `{UUID}`.
    Uuid,

    /// Prefix for a field reference to another entry, e.g. `{REF:T@...}`.
    Reference,

    /// Prefix for a custom string field, e.g. `{S:FieldName}`.
    CustomField,
}

impl Placeholder {
    pub const fn value(self) -> &'static str {
        match self {
            Self::Title => "TITLE",
            Self::UserName => "USERNAME",
            Self::Password => "PASSWORD",
            Self::Url => "URL",
            Self::Notes => "NOTES",
            Self::Uuid => "UUID",
            Self::Reference => "REF:",
            Self::CustomField => "S:",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Placeholder;

    #[test]
    fn values_match_kotlin_placeholders() {
        assert_eq!(Placeholder::Title.value(), "TITLE");
        assert_eq!(Placeholder::UserName.value(), "USERNAME");
        assert_eq!(Placeholder::Password.value(), "PASSWORD");
        assert_eq!(Placeholder::Url.value(), "URL");
        assert_eq!(Placeholder::Notes.value(), "NOTES");
        assert_eq!(Placeholder::Uuid.value(), "UUID");
        assert_eq!(Placeholder::Reference.value(), "REF:");
        assert_eq!(Placeholder::CustomField.value(), "S:");
    }
}
