use super::{DeletedObject, Group, Meta};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseContent {
    pub meta: Meta,
    pub group: Group,
    pub deleted_objects: Vec<DeletedObject>,
}

impl DatabaseContent {
    pub fn new(meta: Meta, group: Group, deleted_objects: Vec<DeletedObject>) -> Self {
        Self {
            meta,
            group,
            deleted_objects,
        }
    }
}
