use indexmap::IndexMap;
use uuid::Uuid;

use crate::{
    constants::PredefinedIcon,
    model::{
        AutoTypeData, BinaryReference, CustomDataValue, Entry, EntryFields, EntryValue, TimeData,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutableEntry {
    pub uuid: Uuid,
    pub icon: PredefinedIcon,
    pub custom_icon_uuid: Option<Uuid>,
    pub foreground_color: Option<String>,
    pub background_color: Option<String>,
    pub override_url: String,
    pub times: Option<TimeData>,
    pub auto_type: Option<AutoTypeData>,
    pub fields: IndexMap<String, EntryValue>,
    pub tags: Vec<String>,
    pub binaries: Vec<BinaryReference>,
    pub history: Vec<Entry>,
    pub custom_data: IndexMap<String, CustomDataValue>,
    pub previous_parent_group: Option<Uuid>,
    pub quality_check: bool,
}

impl MutableEntry {
    pub fn new(uuid: Uuid) -> Self {
        Self {
            uuid,
            icon: PredefinedIcon::Key,
            custom_icon_uuid: None,
            foreground_color: None,
            background_color: None,
            override_url: String::new(),
            times: None,
            auto_type: None,
            fields: IndexMap::new(),
            tags: Vec::new(),
            binaries: Vec::new(),
            history: Vec::new(),
            custom_data: IndexMap::new(),
            previous_parent_group: None,
            quality_check: true,
        }
    }

    pub fn build(self) -> Entry {
        Entry {
            uuid: self.uuid,
            icon: self.icon,
            custom_icon_uuid: self.custom_icon_uuid,
            foreground_color: self.foreground_color,
            background_color: self.background_color,
            override_url: self.override_url,
            times: self.times,
            auto_type: self.auto_type,
            fields: EntryFields::new(self.fields),
            tags: self.tags,
            binaries: self.binaries,
            history: self.history,
            custom_data: self.custom_data,
            previous_parent_group: self.previous_parent_group,
            quality_check: self.quality_check,
        }
    }
}

pub fn build_entry(uuid: Uuid, block: impl FnOnce(&mut MutableEntry)) -> Entry {
    let mut entry = MutableEntry::new(uuid);
    block(&mut entry);
    entry.build()
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::{MutableEntry, build_entry};
    use crate::{
        constants::{BasicField, PredefinedIcon},
        model::EntryValue,
    };

    #[test]
    fn builds_entry_with_kotlin_builder_defaults() {
        let uuid = Uuid::new_v4();

        let entry = build_entry(uuid, |_| {});

        assert_eq!(entry.uuid, uuid);
        assert_eq!(entry.icon, PredefinedIcon::Key);
        assert_eq!(entry.times, None);
        assert!(entry.fields.inner().is_empty());
        assert!(entry.tags.is_empty());
        assert!(entry.quality_check);
    }

    #[test]
    fn applies_mutations_before_building() {
        let uuid = Uuid::new_v4();

        let entry = build_entry(uuid, |entry| {
            entry.icon = PredefinedIcon::Star;
            entry.tags.push("tag".to_owned());
            entry.fields.insert(
                BasicField::Title.key().to_owned(),
                EntryValue::Plain("Title".to_owned()),
            );
        });

        assert_eq!(entry.icon, PredefinedIcon::Star);
        assert_eq!(entry.tags, vec!["tag"]);
        assert_eq!(
            entry.fields.title(),
            Some(&EntryValue::Plain("Title".to_owned()))
        );
    }

    #[test]
    fn mutable_entry_can_be_built_directly() {
        let uuid = Uuid::new_v4();
        let mut entry = MutableEntry::new(uuid);
        entry.override_url = "cmd://open".to_owned();

        assert_eq!(entry.build().override_url, "cmd://open");
    }
}
