use time::OffsetDateTime;
use uuid::Uuid;

use crate::model::{DeletedObject, Entry, Group, TimeData};

use super::super::KeePassDatabase;

impl KeePassDatabase {
    pub fn move_entry(&mut self, uuid: Uuid, parent_group: Uuid) -> &mut KeePassDatabase {
        let Some((previous_parent_uuid, item)) = self
            .get_entry(|entry| entry.uuid == uuid)
            .map(|(parent, entry)| (parent.uuid, entry.clone()))
        else {
            return self;
        };
        if find_group_mut(&mut self.content_mut().group, parent_group).is_none() {
            return self;
        }

        remove_child_entry(&mut self.content_mut().group, uuid);
        let Some(parent) = find_group_mut(&mut self.content_mut().group, parent_group) else {
            return self;
        };
        let mut item = item;
        touch_location(&mut item, previous_parent_uuid);
        parent.entries.push(item);
        self
    }

    pub fn modify_entry(
        &mut self,
        uuid: Uuid,
        block: impl FnOnce(&mut Entry),
    ) -> &mut KeePassDatabase {
        if let Some(entry) = find_entry_mut(&mut self.content_mut().group, uuid) {
            block(entry);
            touch_entry(entry);
        }
        self
    }

    pub fn modify_entries(&mut self, mut block: impl FnMut(&mut Entry)) -> &mut KeePassDatabase {
        modify_entries_in_group(&mut self.content_mut().group, &mut block);
        self
    }

    pub fn remove_entry(&mut self, uuid: Uuid) -> &mut KeePassDatabase {
        remove_child_entry(&mut self.content_mut().group, uuid);
        self.content_mut()
            .deleted_objects
            .push(DeletedObject::new(uuid, OffsetDateTime::now_utc()));
        self
    }
}

impl Entry {
    pub fn with_history(&mut self, block: impl FnOnce(&mut Entry)) -> &mut Entry {
        let mut historic_entry = self.clone();
        historic_entry.history.clear();
        let mut original_history = self.history.clone();

        block(self);
        original_history.push(historic_entry);
        self.history = original_history;
        self
    }
}

fn find_group_mut(group: &mut Group, uuid: Uuid) -> Option<&mut Group> {
    if group.uuid == uuid {
        return Some(group);
    }
    for child in &mut group.groups {
        if let Some(found) = find_group_mut(child, uuid) {
            return Some(found);
        }
    }
    None
}

fn find_entry_mut(group: &mut Group, uuid: Uuid) -> Option<&mut Entry> {
    if let Some(entry) = group.entries.iter_mut().find(|entry| entry.uuid == uuid) {
        return Some(entry);
    }
    for child in &mut group.groups {
        if let Some(found) = find_entry_mut(child, uuid) {
            return Some(found);
        }
    }
    None
}

fn modify_entries_in_group(group: &mut Group, block: &mut impl FnMut(&mut Entry)) {
    for entry in &mut group.entries {
        let before = entry.clone();
        block(entry);
        if *entry != before {
            touch_entry(entry);
        }
    }
    for child in &mut group.groups {
        modify_entries_in_group(child, block);
    }
}

fn remove_child_entry(group: &mut Group, uuid: Uuid) -> bool {
    if let Some(index) = group.entries.iter().position(|entry| entry.uuid == uuid) {
        group.entries.remove(index);
        return true;
    }
    for child in &mut group.groups {
        if remove_child_entry(child, uuid) {
            return true;
        }
    }
    false
}

fn touch_entry(entry: &mut Entry) {
    let now = OffsetDateTime::now_utc();
    match &mut entry.times {
        Some(times) => {
            times.last_access_time = Some(now);
            times.last_modification_time = Some(now);
        }
        None => entry.times = Some(TimeData::create(now)),
    }
}

fn touch_location(entry: &mut Entry, previous_parent_uuid: Uuid) {
    let now = OffsetDateTime::now_utc();
    match &mut entry.times {
        Some(times) => {
            times.location_changed = Some(now);
        }
        None => entry.times = Some(TimeData::create(now)),
    }
    entry.previous_parent_group = Some(previous_parent_uuid);
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::{
        constants::PredefinedIcon,
        crypto::EncryptedValue,
        database::{Credentials, KeePassDatabase},
        model::{Entry, Group, Meta},
    };

    fn credentials() -> Credentials {
        Credentials::from_passphrase(&EncryptedValue::from_string("1").unwrap()).unwrap()
    }

    fn database_with_child_entry() -> (KeePassDatabase, Uuid, Uuid) {
        let mut database =
            KeePassDatabase::create_ver4x("Root", Meta::default(), credentials()).unwrap();
        let child_uuid = Uuid::new_v4();
        let entry_uuid = Uuid::new_v4();
        let mut child = Group::new(child_uuid, "Child");
        child.entries.push(Entry::new(entry_uuid));
        database.content_mut().group.groups.push(child);
        (database, child_uuid, entry_uuid)
    }

    #[test]
    fn modifies_specific_entry_and_touches_times() {
        let (mut database, _, entry_uuid) = database_with_child_entry();

        database.modify_entry(entry_uuid, |entry| {
            entry.icon = PredefinedIcon::Star;
        });

        let entry = database
            .get_entry_by(|entry| entry.uuid == entry_uuid)
            .unwrap();
        assert_eq!(entry.icon, PredefinedIcon::Star);
        assert!(
            entry
                .times
                .as_ref()
                .unwrap()
                .last_modification_time
                .is_some()
        );
    }

    #[test]
    fn modifies_all_changed_entries() {
        let (mut database, _, entry_uuid) = database_with_child_entry();

        database.modify_entries(|entry| {
            if entry.uuid == entry_uuid {
                entry.icon = PredefinedIcon::Star;
            }
        });

        assert_eq!(
            database
                .get_entry_by(|entry| entry.uuid == entry_uuid)
                .unwrap()
                .icon,
            PredefinedIcon::Star
        );
    }

    #[test]
    fn moves_entry_to_new_parent_group() {
        let (mut database, child_uuid, entry_uuid) = database_with_child_entry();
        let root_uuid = database.content().group.uuid;

        database.move_entry(entry_uuid, root_uuid);

        assert!(
            database
                .content()
                .group
                .groups
                .iter()
                .find(|group| group.uuid == child_uuid)
                .unwrap()
                .entries
                .is_empty()
        );
        let moved = database
            .content()
            .group
            .entries
            .iter()
            .find(|entry| entry.uuid == entry_uuid)
            .unwrap();
        assert_eq!(moved.previous_parent_group, Some(child_uuid));
    }

    #[test]
    fn removes_entry_and_records_deleted_object() {
        let (mut database, _, entry_uuid) = database_with_child_entry();

        database.remove_entry(entry_uuid);

        assert!(
            database
                .get_entry_by(|entry| entry.uuid == entry_uuid)
                .is_none()
        );
        assert_eq!(database.content().deleted_objects[0].id, entry_uuid);
    }

    #[test]
    fn entry_with_history_keeps_original_snapshot() {
        let mut entry = Entry::new(Uuid::new_v4());
        entry.icon = PredefinedIcon::Key;

        entry.with_history(|entry| {
            entry.icon = PredefinedIcon::Star;
        });

        assert_eq!(entry.icon, PredefinedIcon::Star);
        assert_eq!(entry.history.len(), 1);
        assert_eq!(entry.history[0].icon, PredefinedIcon::Key);
        assert!(entry.history[0].history.is_empty());
    }
}
