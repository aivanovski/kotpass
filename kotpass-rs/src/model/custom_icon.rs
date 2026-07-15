use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomIcon {
    pub data: Vec<u8>,
    pub name: Option<String>,
    pub last_modified: Option<OffsetDateTime>,
}

impl CustomIcon {
    pub fn new(
        data: impl Into<Vec<u8>>,
        name: Option<String>,
        last_modified: Option<OffsetDateTime>,
    ) -> Self {
        Self {
            data: data.into(),
            name,
            last_modified,
        }
    }
}
