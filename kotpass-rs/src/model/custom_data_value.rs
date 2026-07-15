use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomDataValue {
    pub value: String,
    pub last_modified: Option<OffsetDateTime>,
}

impl CustomDataValue {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            last_modified: None,
        }
    }

    pub fn with_last_modified(value: impl Into<String>, last_modified: OffsetDateTime) -> Self {
        Self {
            value: value.into(),
            last_modified: Some(last_modified),
        }
    }
}
