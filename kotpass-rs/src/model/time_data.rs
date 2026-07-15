use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TimeData {
    pub creation_time: Option<OffsetDateTime>,
    pub last_access_time: Option<OffsetDateTime>,
    pub last_modification_time: Option<OffsetDateTime>,
    pub location_changed: Option<OffsetDateTime>,
    pub expiry_time: Option<OffsetDateTime>,
    pub expires: bool,
    pub usage_count: i32,
}

impl TimeData {
    pub fn create(now: OffsetDateTime) -> Self {
        Self {
            creation_time: Some(now),
            last_access_time: Some(now),
            last_modification_time: Some(now),
            location_changed: Some(now),
            expiry_time: None,
            expires: false,
            usage_count: 0,
        }
    }

    pub fn now() -> Self {
        Self::create(OffsetDateTime::now_utc())
    }
}

impl Default for TimeData {
    fn default() -> Self {
        Self::now()
    }
}
