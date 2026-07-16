use indexmap::IndexMap;
use uuid::Uuid;

use crate::{
    constants::{GroupOverride, PredefinedIcon},
    model::{CustomDataValue, Entry, Group, TimeData},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutableGroup {
    pub uuid: Uuid,
    pub name: String,
    pub notes: String,
    pub icon: PredefinedIcon,
    pub custom_icon_uuid: Option<Uuid>,
    pub times: Option<TimeData>,
    pub expanded: bool,
    pub default_auto_type_sequence: Option<String>,
    pub enable_auto_type: GroupOverride,
    pub enable_searching: GroupOverride,
    pub last_top_visible_entry: Option<Uuid>,
    pub previous_parent_group: Option<Uuid>,
    pub tags: Vec<String>,
    pub groups: Vec<Group>,
    pub entries: Vec<Entry>,
    pub custom_data: IndexMap<String, CustomDataValue>,
}

impl MutableGroup {
    pub fn new(uuid: Uuid) -> Self {
        Self {
            uuid,
            name: String::new(),
            notes: String::new(),
            icon: PredefinedIcon::Folder,
            custom_icon_uuid: None,
            times: None,
            expanded: true,
            default_auto_type_sequence: None,
            enable_auto_type: GroupOverride::Inherit,
            enable_searching: GroupOverride::Inherit,
            last_top_visible_entry: None,
            previous_parent_group: None,
            tags: Vec::new(),
            groups: Vec::new(),
            entries: Vec::new(),
            custom_data: IndexMap::new(),
        }
    }

    pub fn build(self) -> Group {
        Group {
            uuid: self.uuid,
            name: self.name,
            notes: self.notes,
            icon: self.icon,
            custom_icon_uuid: self.custom_icon_uuid,
            times: self.times,
            expanded: self.expanded,
            default_auto_type_sequence: self.default_auto_type_sequence,
            enable_auto_type: self.enable_auto_type,
            enable_searching: self.enable_searching,
            last_top_visible_entry: self.last_top_visible_entry,
            previous_parent_group: self.previous_parent_group,
            tags: self.tags,
            groups: self.groups,
            entries: self.entries,
            custom_data: self.custom_data,
        }
    }
}

pub fn build_group(uuid: Uuid, block: impl FnOnce(&mut MutableGroup)) -> Group {
    let mut group = MutableGroup::new(uuid);
    block(&mut group);
    group.build()
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::{MutableGroup, build_group};
    use crate::{
        builders::build_entry,
        constants::{GroupOverride, PredefinedIcon},
    };

    #[test]
    fn builds_group_with_kotlin_builder_defaults() {
        let uuid = Uuid::new_v4();

        let group = build_group(uuid, |_| {});

        assert_eq!(group.uuid, uuid);
        assert_eq!(group.name, "");
        assert_eq!(group.icon, PredefinedIcon::Folder);
        assert_eq!(group.times, None);
        assert_eq!(group.enable_auto_type, GroupOverride::Inherit);
        assert_eq!(group.enable_searching, GroupOverride::Inherit);
        assert!(group.groups.is_empty());
        assert!(group.entries.is_empty());
    }

    #[test]
    fn applies_mutations_before_building() {
        let uuid = Uuid::new_v4();
        let child_uuid = Uuid::new_v4();
        let entry_uuid = Uuid::new_v4();

        let group = build_group(uuid, |group| {
            group.name = "Root".to_owned();
            group.icon = PredefinedIcon::Star;
            group.enable_searching = GroupOverride::Enabled;
            group.groups.push(build_group(child_uuid, |child| {
                child.name = "Child".to_owned();
            }));
            group.entries.push(build_entry(entry_uuid, |_| {}));
        });

        assert_eq!(group.name, "Root");
        assert_eq!(group.icon, PredefinedIcon::Star);
        assert_eq!(group.enable_searching, GroupOverride::Enabled);
        assert_eq!(group.groups[0].uuid, child_uuid);
        assert_eq!(group.entries[0].uuid, entry_uuid);
    }

    #[test]
    fn mutable_group_can_be_built_directly() {
        let uuid = Uuid::new_v4();
        let mut group = MutableGroup::new(uuid);
        group.notes = "notes".to_owned();

        assert_eq!(group.build().notes, "notes");
    }
}
