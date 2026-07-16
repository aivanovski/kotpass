use time::OffsetDateTime;

use super::super::{Credentials, KeePassDatabase};

impl KeePassDatabase {
    pub fn modify_credentials(
        &mut self,
        block: impl FnOnce(&Credentials) -> Credentials,
    ) -> &mut KeePassDatabase {
        let new_credentials = block(self.credentials());
        let passphrase_changed =
            passphrase_hash(self.credentials()) != passphrase_hash(&new_credentials);

        match self {
            Self::Ver3x { credentials, .. } | Self::Ver4x { credentials, .. } => {
                *credentials = new_credentials;
            }
        }

        if passphrase_changed {
            self.content_mut().meta.master_key_changed = Some(OffsetDateTime::now_utc());
        }

        self
    }
}

fn passphrase_hash(credentials: &Credentials) -> Option<Vec<u8>> {
    credentials
        .passphrase
        .as_ref()
        .map(|passphrase| passphrase.get_hash())
}

#[cfg(test)]
mod tests {
    use crate::{
        crypto::EncryptedValue,
        database::{Credentials, KeePassDatabase},
        model::Meta,
    };

    fn credentials(passphrase: &str) -> Credentials {
        Credentials::from_passphrase(&EncryptedValue::from_string(passphrase).unwrap()).unwrap()
    }

    #[test]
    fn modifies_credentials() {
        let mut database =
            KeePassDatabase::create_ver4x("Root", Meta::default(), credentials("1")).unwrap();
        let new_credentials = credentials("2");

        database.modify_credentials(|_| new_credentials.clone());

        assert_eq!(
            database
                .credentials()
                .passphrase
                .as_ref()
                .unwrap()
                .get_hash(),
            new_credentials.passphrase.as_ref().unwrap().get_hash()
        );
    }

    #[test]
    fn updates_master_key_changed_when_passphrase_changes() {
        let mut database =
            KeePassDatabase::create_ver4x("Root", Meta::default(), credentials("1")).unwrap();
        database.content_mut().meta.master_key_changed = None;

        database.modify_credentials(|_| credentials("2"));

        assert!(database.content().meta.master_key_changed.is_some());
    }

    #[test]
    fn leaves_master_key_changed_when_passphrase_is_same() {
        let mut database =
            KeePassDatabase::create_ver4x("Root", Meta::default(), credentials("1")).unwrap();
        database.content_mut().meta.master_key_changed = None;

        database.modify_credentials(|current| current.clone());

        assert_eq!(database.content().meta.master_key_changed, None);
    }
}
