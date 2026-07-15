use indexmap::IndexMap;
use uuid::Uuid;

use crate::constants::{BasicField, PredefinedIcon};

use super::{
    AutoTypeData, BinaryReference, CustomDataValue, DatabaseElement, EntryFields, EntryValue,
    TimeData,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub uuid: Uuid,
    pub icon: PredefinedIcon,
    pub custom_icon_uuid: Option<Uuid>,
    pub foreground_color: Option<String>,
    pub background_color: Option<String>,
    pub override_url: String,
    pub times: Option<TimeData>,
    pub auto_type: Option<AutoTypeData>,
    pub fields: EntryFields,
    pub tags: Vec<String>,
    pub binaries: Vec<BinaryReference>,
    pub history: Vec<Entry>,
    pub custom_data: IndexMap<String, CustomDataValue>,
    pub previous_parent_group: Option<Uuid>,
    pub quality_check: bool,
}

impl Entry {
    pub fn new(uuid: Uuid) -> Self {
        Self {
            uuid,
            icon: PredefinedIcon::Key,
            custom_icon_uuid: None,
            foreground_color: None,
            background_color: None,
            override_url: String::new(),
            times: Some(TimeData::now()),
            auto_type: None,
            fields: EntryFields::default(),
            tags: Vec::new(),
            binaries: Vec::new(),
            history: Vec::new(),
            custom_data: IndexMap::new(),
            previous_parent_group: None,
            quality_check: true,
        }
    }

    pub fn get(&self, field: BasicField) -> Option<&EntryValue> {
        self.fields.get_basic(field)
    }
}

impl DatabaseElement for Entry {
    fn uuid(&self) -> Uuid {
        self.uuid
    }

    fn times(&self) -> Option<&TimeData> {
        self.times.as_ref()
    }

    fn icon(&self) -> PredefinedIcon {
        self.icon
    }

    fn custom_icon_uuid(&self) -> Option<Uuid> {
        self.custom_icon_uuid
    }

    fn tags(&self) -> &[String] {
        &self.tags
    }
}
