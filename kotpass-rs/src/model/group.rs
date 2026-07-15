use indexmap::IndexMap;
use uuid::Uuid;

use crate::constants::{GroupOverride, PredefinedIcon};

use super::{CustomDataValue, DatabaseElement, DatabaseElementRef, Entry, TimeData};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
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

impl Group {
    pub fn new(uuid: Uuid, name: impl Into<String>) -> Self {
        Self {
            uuid,
            name: name.into(),
            notes: String::new(),
            icon: PredefinedIcon::Folder,
            custom_icon_uuid: None,
            times: Some(TimeData::now()),
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

    pub fn create_recycle_bin(name: impl Into<String>) -> Self {
        let mut group = Self::new(Uuid::new_v4(), name);
        group.icon = PredefinedIcon::TrashBin;
        group.enable_searching = GroupOverride::Disabled;
        group.enable_auto_type = GroupOverride::Disabled;
        group
    }

    pub fn traverse<'a>(&'a self, mut block: impl FnMut(DatabaseElementRef<'a>)) {
        let mut stack = vec![self];

        while let Some(current) = stack.pop() {
            block(DatabaseElementRef::Group(current));

            for entry in &current.entries {
                block(DatabaseElementRef::Entry(entry));
            }
            for group in &current.groups {
                stack.push(group);
            }
        }
    }

    pub fn find_child_group(
        &self,
        recycle_bin_uuid: Option<Uuid>,
        predicate: impl Fn(&Group) -> bool,
    ) -> Option<(&Group, &Group)> {
        let mut stack: Vec<(&Group, &Group)> = self
            .groups
            .iter()
            .filter(|group| recycle_bin_uuid.is_none_or(|uuid| group.uuid != uuid))
            .map(|group| (self, group))
            .collect();

        while let Some((parent, current)) = stack.pop() {
            if predicate(current) {
                return Some((parent, current));
            }

            stack.extend(
                current
                    .groups
                    .iter()
                    .filter(|group| recycle_bin_uuid.is_none_or(|uuid| group.uuid != uuid))
                    .map(|group| (current, group)),
            );
        }

        None
    }

    pub fn find_child_entry(
        &self,
        use_group_override: bool,
        recycle_bin_uuid: Option<Uuid>,
        predicate: impl Fn(&Entry) -> bool,
    ) -> Option<(&Group, &Entry)> {
        let mut stack = vec![(self, true)];

        while let Some((current, parent_search_enabled)) = stack.pop() {
            let search_enabled = current.search_enabled(parent_search_enabled);

            if !use_group_override || search_enabled {
                if let Some(entry) = current.entries.iter().find(|entry| predicate(entry)) {
                    return Some((current, entry));
                }
            }

            stack.extend(
                current
                    .groups
                    .iter()
                    .filter(|group| recycle_bin_uuid.is_none_or(|uuid| group.uuid != uuid))
                    .map(|group| (group, search_enabled)),
            );
        }

        None
    }

    pub fn find_child_entries(
        &self,
        use_group_override: bool,
        recycle_bin_uuid: Option<Uuid>,
        predicate: impl Fn(&Entry) -> bool,
    ) -> Vec<(&Group, Vec<&Entry>)> {
        let mut result = Vec::new();
        let mut stack = vec![(self, true)];

        while let Some((current, parent_search_enabled)) = stack.pop() {
            let search_enabled = current.search_enabled(parent_search_enabled);

            if !use_group_override || search_enabled {
                let found: Vec<_> = current
                    .entries
                    .iter()
                    .filter(|entry| predicate(entry))
                    .collect();

                if !found.is_empty() {
                    result.push((current, found));
                }
            }

            stack.extend(
                current
                    .groups
                    .iter()
                    .filter(|group| recycle_bin_uuid.is_none_or(|uuid| group.uuid != uuid))
                    .map(|group| (group, search_enabled)),
            );
        }

        result
    }

    fn search_enabled(&self, parent_search_enabled: bool) -> bool {
        match self.enable_searching {
            GroupOverride::Inherit => parent_search_enabled,
            GroupOverride::Enabled => true,
            GroupOverride::Disabled => false,
        }
    }
}

impl DatabaseElement for Group {
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

#[cfg(test)]
mod tests {
    use crate::{
        constants::GroupOverride,
        model::{Entry, Group},
    };
    use uuid::Uuid;

    #[test]
    fn finds_child_entry_respecting_group_search_override() {
        let entry = Entry::new(Uuid::new_v4());
        let mut child = Group::new(Uuid::new_v4(), "child");
        child.enable_searching = GroupOverride::Disabled;
        child.entries.push(entry);
        let entry_uuid = child.entries[0].uuid;
        let mut root = Group::new(Uuid::new_v4(), "root");
        root.groups.push(child);

        assert!(
            root.find_child_entry(true, None, |candidate| candidate.uuid == entry_uuid)
                .is_none()
        );
        assert!(
            root.find_child_entry(false, None, |candidate| candidate.uuid == entry_uuid)
                .is_some()
        );
    }
}
