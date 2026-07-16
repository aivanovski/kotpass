use indexmap::IndexSet;
use uuid::Uuid;

use crate::model::{CustomIcon, Entry, Group};

use super::super::KeePassDatabase;

impl KeePassDatabase {
    pub fn modify_custom_icons(
        &mut self,
        block: impl FnOnce(&mut indexmap::IndexMap<Uuid, CustomIcon>),
    ) -> &mut KeePassDatabase {
        let old_keys: IndexSet<Uuid> = self.content().meta.custom_icons.keys().copied().collect();
        block(&mut self.content_mut().meta.custom_icons);
        let new_keys: IndexSet<Uuid> = self.content().meta.custom_icons.keys().copied().collect();
        let removed: IndexSet<Uuid> = old_keys.difference(&new_keys).copied().collect();

        if !removed.is_empty() {
            clear_removed_group_icon_refs(&mut self.content_mut().group, &removed);
        }

        self
    }
}

fn clear_removed_group_icon_refs(group: &mut Group, removed: &IndexSet<Uuid>) {
    if group
        .custom_icon_uuid
        .is_some_and(|uuid| removed.contains(&uuid))
    {
        group.custom_icon_uuid = None;
    }
    for entry in &mut group.entries {
        clear_removed_entry_icon_refs(entry, removed);
    }
    for child in &mut group.groups {
        clear_removed_group_icon_refs(child, removed);
    }
}

fn clear_removed_entry_icon_refs(entry: &mut Entry, removed: &IndexSet<Uuid>) {
    if entry
        .custom_icon_uuid
        .is_some_and(|uuid| removed.contains(&uuid))
    {
        entry.custom_icon_uuid = None;
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::{
        crypto::EncryptedValue,
        database::{Credentials, KeePassDatabase},
        model::{CustomIcon, Entry, Group, Meta},
    };

    fn credentials() -> Credentials {
        Credentials::from_passphrase(&EncryptedValue::from_string("1").unwrap()).unwrap()
    }

    #[test]
    fn modifies_custom_icon_map() {
        let mut database =
            KeePassDatabase::create_ver4x("Root", Meta::default(), credentials()).unwrap();
        let icon_uuid = Uuid::new_v4();

        database.modify_custom_icons(|icons| {
            icons.insert(icon_uuid, CustomIcon::new([1], None, None));
        });

        assert!(
            database
                .content()
                .meta
                .custom_icons
                .contains_key(&icon_uuid)
        );
    }

    #[test]
    fn clears_removed_custom_icon_references_from_groups_and_entries() {
        let mut database =
            KeePassDatabase::create_ver4x("Root", Meta::default(), credentials()).unwrap();
        let removed_uuid = Uuid::new_v4();
        let kept_uuid = Uuid::new_v4();
        database.modify_custom_icons(|icons| {
            icons.insert(removed_uuid, CustomIcon::new([1], None, None));
            icons.insert(kept_uuid, CustomIcon::new([2], None, None));
        });
        database.content_mut().group.custom_icon_uuid = Some(removed_uuid);
        let mut child = Group::new(Uuid::new_v4(), "Child");
        child.custom_icon_uuid = Some(kept_uuid);
        let mut entry = Entry::new(Uuid::new_v4());
        entry.custom_icon_uuid = Some(removed_uuid);
        child.entries.push(entry);
        database.content_mut().group.groups.push(child);

        database.modify_custom_icons(|icons| {
            icons.shift_remove(&removed_uuid);
        });

        assert_eq!(database.content().group.custom_icon_uuid, None);
        assert_eq!(
            database.content().group.groups[0].custom_icon_uuid,
            Some(kept_uuid)
        );
        assert_eq!(
            database.content().group.groups[0].entries[0].custom_icon_uuid,
            None
        );
    }
}
