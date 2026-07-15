#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WantedField {
    Title,
    UserName,
    Password,
    Url,
    Notes,
    Uuid,
}

impl WantedField {
    pub const ALL: [Self; 6] = [
        Self::Title,
        Self::UserName,
        Self::Password,
        Self::Url,
        Self::Notes,
        Self::Uuid,
    ];

    pub const fn key(self) -> &'static str {
        match self {
            Self::Title => "T",
            Self::UserName => "U",
            Self::Password => "P",
            Self::Url => "A",
            Self::Notes => "N",
            Self::Uuid => "I",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|field| field.key().eq_ignore_ascii_case(key))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchIn {
    Title,
    UserName,
    Password,
    Url,
    Notes,
    Uuid,
    Other,
}

impl SearchIn {
    pub const ALL: [Self; 7] = [
        Self::Title,
        Self::UserName,
        Self::Password,
        Self::Url,
        Self::Notes,
        Self::Uuid,
        Self::Other,
    ];

    pub const fn key(self) -> &'static str {
        match self {
            Self::Title => "T",
            Self::UserName => "U",
            Self::Password => "P",
            Self::Url => "A",
            Self::Notes => "N",
            Self::Uuid => "I",
            Self::Other => "O",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|field| field.key().eq_ignore_ascii_case(key))
    }
}

#[cfg(test)]
mod tests {
    use super::{SearchIn, WantedField};

    #[test]
    fn wanted_field_keys_match_kotlin_values() {
        assert_eq!(WantedField::Title.key(), "T");
        assert_eq!(WantedField::UserName.key(), "U");
        assert_eq!(WantedField::Password.key(), "P");
        assert_eq!(WantedField::Url.key(), "A");
        assert_eq!(WantedField::Notes.key(), "N");
        assert_eq!(WantedField::Uuid.key(), "I");
    }

    #[test]
    fn wanted_field_lookup_is_case_insensitive() {
        assert_eq!(WantedField::from_key("t"), Some(WantedField::Title));
        assert_eq!(WantedField::from_key("I"), Some(WantedField::Uuid));
        assert_eq!(WantedField::from_key("x"), None);
    }

    #[test]
    fn search_in_keys_match_kotlin_values() {
        assert_eq!(SearchIn::Title.key(), "T");
        assert_eq!(SearchIn::UserName.key(), "U");
        assert_eq!(SearchIn::Password.key(), "P");
        assert_eq!(SearchIn::Url.key(), "A");
        assert_eq!(SearchIn::Notes.key(), "N");
        assert_eq!(SearchIn::Uuid.key(), "I");
        assert_eq!(SearchIn::Other.key(), "O");
    }

    #[test]
    fn search_in_lookup_is_case_insensitive() {
        assert_eq!(SearchIn::from_key("o"), Some(SearchIn::Other));
        assert_eq!(SearchIn::from_key("A"), Some(SearchIn::Url));
        assert_eq!(SearchIn::from_key("x"), None);
    }
}
