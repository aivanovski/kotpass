use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{
    constants::defaults,
    model::{DatabaseContent, Entry, Group},
};

use super::super::KeePassDatabase;

impl KeePassDatabase {
    pub fn modify_content(
        &mut self,
        block: impl FnOnce(&mut DatabaseContent),
    ) -> &mut KeePassDatabase {
        block(self.content_mut());
        self
    }

    pub fn with_recycle_bin(
        &mut self,
        block: impl FnOnce(&mut KeePassDatabase, Uuid),
    ) -> &mut KeePassDatabase {
        let recycle_bin_uuid = self.content().meta.recycle_bin_uuid;
        let recycle_bin_uuid = match recycle_bin_uuid {
            Some(uuid) if !uuid.is_nil() => uuid,
            _ => self.create_recycle_bin(),
        };

        block(self, recycle_bin_uuid);
        self
    }

    pub fn cleanup_history(&mut self, reference: OffsetDateTime) -> &mut KeePassDatabase {
        let maintenance_period =
            Duration::days(i64::from(self.content().meta.maintenance_history_days));
        let history_max_items = self.content().meta.history_max_items;

        cleanup_group_history(
            &mut self.content_mut().group,
            reference,
            maintenance_period,
            history_max_items,
        );
        self
    }

    pub fn cleanup_history_now(&mut self) -> &mut KeePassDatabase {
        self.cleanup_history(OffsetDateTime::now_utc())
    }

    fn create_recycle_bin(&mut self) -> Uuid {
        let recycle_bin = Group::create_recycle_bin(defaults::RECYCLE_BIN_NAME);
        let recycle_bin_uuid = recycle_bin.uuid;
        let now = OffsetDateTime::now_utc();
        let content = self.content_mut();

        content.meta.recycle_bin_enabled = true;
        content.meta.recycle_bin_uuid = Some(recycle_bin_uuid);
        content.meta.recycle_bin_changed = Some(now);
        content.group.groups.push(recycle_bin);

        recycle_bin_uuid
    }
}

fn cleanup_group_history(
    group: &mut Group,
    reference: OffsetDateTime,
    maintenance_period: Duration,
    history_max_items: i32,
) {
    for child in &mut group.groups {
        cleanup_group_history(child, reference, maintenance_period, history_max_items);
    }
    for entry in &mut group.entries {
        cleanup_entry_history(entry, reference, maintenance_period, history_max_items);
    }
}

fn cleanup_entry_history(
    entry: &mut Entry,
    reference: OffsetDateTime,
    maintenance_period: Duration,
    history_max_items: i32,
) {
    entry.history.retain(|historical_entry| {
        historical_entry
            .times
            .as_ref()
            .and_then(|times| times.last_modification_time)
            .is_none_or(|last_modification_time| {
                reference - last_modification_time < maintenance_period
            })
    });

    if history_max_items >= 0 {
        let max_items = history_max_items as usize;
        if entry.history.len() > max_items {
            let remove_count = entry.history.len() - max_items;
            entry.history.drain(..remove_count);
        }
    }
}

#[cfg(test)]
mod tests {
    use time::{Duration, OffsetDateTime};
    use uuid::Uuid;

    use crate::{
        crypto::EncryptedValue,
        database::{Credentials, KeePassDatabase},
        model::{Entry, Meta, TimeData},
    };

    fn credentials() -> Credentials {
        Credentials::from_passphrase(&EncryptedValue::from_string("1").unwrap()).unwrap()
    }

    #[test]
    fn modifies_content_in_place() {
        let mut database =
            KeePassDatabase::create_ver4x("Root", Meta::default(), credentials()).unwrap();

        database.modify_content(|content| {
            content.meta.name = "Database".to_owned();
        });

        assert_eq!(database.content().meta.name, "Database");
    }

    #[test]
    fn creates_recycle_bin_when_missing() {
        let mut database =
            KeePassDatabase::create_ver4x("Root", Meta::default(), credentials()).unwrap();
        let mut callback_uuid = Uuid::nil();

        database.with_recycle_bin(|_, recycle_bin_uuid| {
            callback_uuid = recycle_bin_uuid;
        });

        assert_eq!(
            database.content().meta.recycle_bin_uuid,
            Some(callback_uuid)
        );
        assert!(database.content().meta.recycle_bin_enabled);
        assert!(
            database
                .content()
                .group
                .groups
                .iter()
                .any(|group| group.uuid == callback_uuid && group.name == "Recycle Bin")
        );
    }

    #[test]
    fn reuses_existing_recycle_bin() {
        let recycle_bin_uuid = Uuid::new_v4();
        let mut meta = Meta::default();
        meta.recycle_bin_uuid = Some(recycle_bin_uuid);
        let mut database = KeePassDatabase::create_ver4x("Root", meta, credentials()).unwrap();

        database.with_recycle_bin(|_, uuid| {
            assert_eq!(uuid, recycle_bin_uuid);
        });

        assert!(database.content().group.groups.is_empty());
    }

    #[test]
    fn cleans_outdated_history_and_applies_max_items() {
        let now = OffsetDateTime::now_utc();
        let mut meta = Meta::default();
        meta.maintenance_history_days = 10;
        meta.history_max_items = 1;
        let mut database = KeePassDatabase::create_ver4x("Root", meta, credentials()).unwrap();
        let mut entry = Entry::new(Uuid::new_v4());
        entry
            .history
            .push(historical_entry(now - Duration::days(20)));
        entry
            .history
            .push(historical_entry(now - Duration::days(2)));
        entry
            .history
            .push(historical_entry(now - Duration::days(1)));
        let kept_uuid = entry.history[2].uuid;
        database.content_mut().group.entries.push(entry);

        database.cleanup_history(now);

        let history = &database.content().group.entries[0].history;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].uuid, kept_uuid);
    }

    fn historical_entry(last_modification_time: OffsetDateTime) -> Entry {
        let mut entry = Entry::new(Uuid::new_v4());
        entry.times = Some(TimeData {
            last_modification_time: Some(last_modification_time),
            ..TimeData::now()
        });
        entry
    }
}
