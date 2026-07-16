use time::OffsetDateTime;
use uuid::Uuid;

use crate::model::{DeletedObject, Group, TimeData};

use super::super::KeePassDatabase;

impl KeePassDatabase {
    pub fn move_group(&mut self, uuid: Uuid, parent_group: Uuid) -> &mut KeePassDatabase {
        if self.content().group.uuid == uuid {
            return self;
        }
        let Some((previous_parent_uuid, item)) = self
            .get_group(|group| group.uuid == uuid)
            .map(|(parent, group)| (parent.map(|parent| parent.uuid), group.clone()))
        else {
            return self;
        };
        if find_group_mut(&mut self.content_mut().group, parent_group).is_none() {
            return self;
        }

        remove_child_group(&mut self.content_mut().group, uuid);
        let Some(parent) = find_group_mut(&mut self.content_mut().group, parent_group) else {
            return self;
        };
        let mut item = item;
        touch_location(&mut item, previous_parent_uuid);
        parent.groups.push(item);
        self
    }

    pub fn modify_parent_group(&mut self, block: impl FnOnce(&mut Group)) -> &mut KeePassDatabase {
        let root_uuid = self.content().group.uuid;
        self.modify_group(root_uuid, block)
    }

    pub fn modify_group(
        &mut self,
        uuid: Uuid,
        block: impl FnOnce(&mut Group),
    ) -> &mut KeePassDatabase {
        if let Some(group) = find_group_mut(&mut self.content_mut().group, uuid) {
            block(group);
            touch_group(group);
        }
        self
    }

    pub fn modify_groups(&mut self, mut block: impl FnMut(&mut Group)) -> &mut KeePassDatabase {
        modify_groups_in_group(&mut self.content_mut().group, &mut block);
        self
    }

    pub fn remove_group(&mut self, uuid: Uuid) -> &mut KeePassDatabase {
        let now = OffsetDateTime::now_utc();
        let deleted_objects = self
            .find_group_child_ids(uuid)
            .into_iter()
            .chain([uuid])
            .map(|id| DeletedObject::new(id, now))
            .collect();

        remove_child_group(&mut self.content_mut().group, uuid);
        self.content_mut().deleted_objects = deleted_objects;
        self
    }

    fn find_group_child_ids(&self, uuid: Uuid) -> Vec<Uuid> {
        let mut ids = Vec::new();
        if let Some((_, found_group)) = self.get_group(|group| group.uuid == uuid) {
            ids.extend(found_group.entries.iter().map(|entry| entry.uuid));
            let mut stack: Vec<&Group> = found_group.groups.iter().collect();
            while let Some(group) = stack.pop() {
                ids.push(group.uuid);
                ids.extend(group.entries.iter().map(|entry| entry.uuid));
                stack.extend(&group.groups);
            }
        }
        ids
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

fn remove_child_group(group: &mut Group, uuid: Uuid) -> bool {
    if let Some(index) = group.groups.iter().position(|group| group.uuid == uuid) {
        group.groups.remove(index);
        return true;
    }
    for child in &mut group.groups {
        if remove_child_group(child, uuid) {
            return true;
        }
    }
    false
}

fn modify_groups_in_group(group: &mut Group, block: &mut impl FnMut(&mut Group)) {
    let before = group.clone();
    block(group);
    if *group != before {
        touch_group(group);
    }
    for child in &mut group.groups {
        modify_groups_in_group(child, block);
    }
}

fn touch_group(group: &mut Group) {
    let now = OffsetDateTime::now_utc();
    match &mut group.times {
        Some(times) => {
            times.last_access_time = Some(now);
            times.last_modification_time = Some(now);
        }
        None => group.times = Some(TimeData::create(now)),
    }
}

fn touch_location(group: &mut Group, previous_parent_uuid: Option<Uuid>) {
    let now = OffsetDateTime::now_utc();
    match &mut group.times {
        Some(times) => {
            times.location_changed = Some(now);
        }
        None => group.times = Some(TimeData::create(now)),
    }
    group.previous_parent_group = previous_parent_uuid;
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

    fn database_with_groups() -> (KeePassDatabase, Uuid, Uuid, Uuid) {
        let mut database =
            KeePassDatabase::create_ver4x("Root", Meta::default(), credentials()).unwrap();
        let parent_uuid = Uuid::new_v4();
        let child_uuid = Uuid::new_v4();
        let entry_uuid = Uuid::new_v4();
        let mut parent = Group::new(parent_uuid, "Parent");
        let mut child = Group::new(child_uuid, "Child");
        child.entries.push(Entry::new(entry_uuid));
        parent.groups.push(child);
        database.content_mut().group.groups.push(parent);
        (database, parent_uuid, child_uuid, entry_uuid)
    }

    #[test]
    fn modifies_parent_group() {
        let (mut database, _, _, _) = database_with_groups();

        database.modify_parent_group(|group| {
            group.name = "Root2".to_owned();
        });

        assert_eq!(database.content().group.name, "Root2");
    }

    #[test]
    fn modifies_specific_group_and_touches_times() {
        let (mut database, parent_uuid, _, _) = database_with_groups();

        database.modify_group(parent_uuid, |group| {
            group.icon = PredefinedIcon::Star;
        });

        let group = database
            .get_group_by(|group| group.uuid == parent_uuid)
            .unwrap();
        assert_eq!(group.icon, PredefinedIcon::Star);
        assert!(
            group
                .times
                .as_ref()
                .unwrap()
                .last_modification_time
                .is_some()
        );
    }

    #[test]
    fn modifies_all_changed_groups() {
        let (mut database, _, child_uuid, _) = database_with_groups();

        database.modify_groups(|group| {
            if group.uuid == child_uuid {
                group.name = "Updated".to_owned();
            }
        });

        assert_eq!(
            database
                .get_group_by(|group| group.uuid == child_uuid)
                .unwrap()
                .name,
            "Updated"
        );
    }

    #[test]
    fn moves_group_to_new_parent() {
        let (mut database, parent_uuid, child_uuid, _) = database_with_groups();
        let root_uuid = database.content().group.uuid;

        database.move_group(child_uuid, root_uuid);

        assert!(
            database
                .get_group_by(|group| group.uuid == parent_uuid)
                .unwrap()
                .groups
                .is_empty()
        );
        let moved = database
            .content()
            .group
            .groups
            .iter()
            .find(|group| group.uuid == child_uuid)
            .unwrap();
        assert_eq!(moved.previous_parent_group, Some(parent_uuid));
    }

    #[test]
    fn removes_group_and_records_child_deleted_objects() {
        let (mut database, parent_uuid, child_uuid, entry_uuid) = database_with_groups();

        database.remove_group(parent_uuid);

        assert!(
            database
                .get_group_by(|group| group.uuid == parent_uuid)
                .is_none()
        );
        let deleted_ids: Vec<_> = database
            .content()
            .deleted_objects
            .iter()
            .map(|object| object.id)
            .collect();
        assert!(deleted_ids.contains(&parent_uuid));
        assert!(deleted_ids.contains(&child_uuid));
        assert!(deleted_ids.contains(&entry_uuid));
    }
}
