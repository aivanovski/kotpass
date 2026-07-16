use time::OffsetDateTime;

use crate::model::Meta;

use super::super::KeePassDatabase;

impl KeePassDatabase {
    pub fn modify_meta(&mut self, block: impl FnOnce(&mut Meta)) -> &mut KeePassDatabase {
        let compare_with = self.content().meta.clone();
        block(&mut self.content_mut().meta);
        self.content_mut().meta.update_timestamps(&compare_with);
        self
    }
}

impl Meta {
    pub fn update_timestamps(&mut self, compare_with: &Meta) {
        let now = OffsetDateTime::now_utc();

        self.settings_changed = if self.recycle_bin_enabled != compare_with.recycle_bin_enabled
            || self.maintenance_history_days != compare_with.maintenance_history_days
            || self.memory_protection != compare_with.memory_protection
            || self.history_max_items != compare_with.history_max_items
            || self.history_max_size != compare_with.history_max_size
            || self.master_key_change_rec != compare_with.master_key_change_rec
            || self.master_key_change_force != compare_with.master_key_change_force
        {
            Some(now)
        } else {
            compare_with.settings_changed
        };
        self.name_changed = if self.name != compare_with.name {
            Some(now)
        } else {
            compare_with.name_changed
        };
        self.description_changed = if self.description != compare_with.description {
            Some(now)
        } else {
            compare_with.description_changed
        };
        self.default_user_changed = if self.default_user != compare_with.default_user {
            Some(now)
        } else {
            compare_with.default_user_changed
        };
        self.recycle_bin_changed = if self.recycle_bin_uuid != compare_with.recycle_bin_uuid {
            Some(now)
        } else {
            compare_with.recycle_bin_changed
        };
        self.entry_templates_group_changed =
            if self.entry_templates_group != compare_with.entry_templates_group {
                Some(now)
            } else {
                compare_with.entry_templates_group_changed
            };
    }
}

#[cfg(test)]
mod tests {
    use time::OffsetDateTime;
    use uuid::Uuid;

    use crate::{
        crypto::EncryptedValue,
        database::{Credentials, KeePassDatabase},
        model::Meta,
    };

    fn credentials() -> Credentials {
        Credentials::from_passphrase(&EncryptedValue::from_string("1").unwrap()).unwrap()
    }

    #[test]
    fn modifies_meta() {
        let mut database =
            KeePassDatabase::create_ver4x("Root", Meta::default(), credentials()).unwrap();

        database.modify_meta(|meta| {
            meta.name = "Database".to_owned();
        });

        assert_eq!(database.content().meta.name, "Database");
        assert!(database.content().meta.name_changed.is_some());
    }

    #[test]
    fn updates_only_changed_timestamps() {
        let timestamp = OffsetDateTime::UNIX_EPOCH;
        let mut meta = Meta::default();
        meta.settings_changed = Some(timestamp);
        meta.name_changed = Some(timestamp);
        meta.description_changed = Some(timestamp);
        let mut database = KeePassDatabase::create_ver4x("Root", meta, credentials()).unwrap();

        database.modify_meta(|meta| {
            meta.maintenance_history_days += 1;
            meta.description = "changed".to_owned();
        });

        assert_ne!(database.content().meta.settings_changed, Some(timestamp));
        assert_eq!(database.content().meta.name_changed, Some(timestamp));
        assert_ne!(database.content().meta.description_changed, Some(timestamp));
    }

    #[test]
    fn updates_recycle_bin_and_template_group_timestamps() {
        let timestamp = OffsetDateTime::UNIX_EPOCH;
        let mut meta = Meta::default();
        meta.recycle_bin_changed = Some(timestamp);
        meta.entry_templates_group_changed = Some(timestamp);
        let mut database = KeePassDatabase::create_ver4x("Root", meta, credentials()).unwrap();

        database.modify_meta(|meta| {
            meta.recycle_bin_uuid = Some(Uuid::new_v4());
            meta.entry_templates_group = Some(Uuid::new_v4());
        });

        assert_ne!(database.content().meta.recycle_bin_changed, Some(timestamp));
        assert_ne!(
            database.content().meta.entry_templates_group_changed,
            Some(timestamp)
        );
    }
}
