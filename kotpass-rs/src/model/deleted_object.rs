use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeletedObject {
    pub id: Uuid,
    pub deletion_time: OffsetDateTime,
}

impl DeletedObject {
    pub fn new(id: Uuid, deletion_time: OffsetDateTime) -> Self {
        Self { id, deletion_time }
    }
}
