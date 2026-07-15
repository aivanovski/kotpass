/// Basic fields which should be added to every entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BasicField {
    Title,
    UserName,
    Password,
    Url,
    Notes,
}

impl BasicField {
    pub const ALL: [Self; 5] = [
        Self::Title,
        Self::UserName,
        Self::Password,
        Self::Url,
        Self::Notes,
    ];

    pub const KEYS: [&'static str; 5] = ["Title", "UserName", "Password", "URL", "Notes"];

    pub const fn key(self) -> &'static str {
        match self {
            Self::Title => "Title",
            Self::UserName => "UserName",
            Self::Password => "Password",
            Self::Url => "URL",
            Self::Notes => "Notes",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "Title" => Some(Self::Title),
            "UserName" => Some(Self::UserName),
            "Password" => Some(Self::Password),
            "URL" => Some(Self::Url),
            "Notes" => Some(Self::Notes),
            _ => None,
        }
    }

    pub fn is_basic_key(key: &str) -> bool {
        Self::from_key(key).is_some()
    }
}

impl AsRef<str> for BasicField {
    fn as_ref(&self) -> &str {
        self.key()
    }
}

#[cfg(test)]
mod tests {
    use super::BasicField;

    #[test]
    fn keys_match_kotlin_values() {
        assert_eq!(BasicField::Title.key(), "Title");
        assert_eq!(BasicField::UserName.key(), "UserName");
        assert_eq!(BasicField::Password.key(), "Password");
        assert_eq!(BasicField::Url.key(), "URL");
        assert_eq!(BasicField::Notes.key(), "Notes");
    }

    #[test]
    fn keys_match_kotlin_enum_order() {
        let keys: Vec<_> = BasicField::ALL.iter().map(|field| field.key()).collect();
        assert_eq!(keys, BasicField::KEYS);
    }

    #[test]
    fn parses_basic_field_keys() {
        assert_eq!(BasicField::from_key("Title"), Some(BasicField::Title));
        assert_eq!(BasicField::from_key("UserName"), Some(BasicField::UserName));
        assert_eq!(BasicField::from_key("Password"), Some(BasicField::Password));
        assert_eq!(BasicField::from_key("URL"), Some(BasicField::Url));
        assert_eq!(BasicField::from_key("Notes"), Some(BasicField::Notes));
        assert_eq!(BasicField::from_key("Url"), None);
        assert_eq!(BasicField::from_key("Custom"), None);
    }

    #[test]
    fn detects_basic_keys() {
        assert!(BasicField::is_basic_key("Title"));
        assert!(BasicField::is_basic_key("URL"));
        assert!(!BasicField::is_basic_key("Custom"));
    }
}
