use std::time::Instant;

use uuid::Uuid;

use crate::{
    constants::GroupOverride,
    crypto::{
        BaseKdfProvider, KdfParameters as CryptoKdfParameters, KdfProvider, byte_array::clear,
    },
    error::{CryptoError, FormatError},
    model::{DatabaseContent, DatabaseElementRef, Entry, Group, Meta},
};

use super::{
    Credentials,
    header::{DatabaseHeader, DatabaseInnerHeader},
};

pub const MIN_SUPPORTED_VERSION: u32 = 3;
pub const MAX_SUPPORTED_VERSION: u32 = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeePassDatabase {
    Ver3x {
        credentials: Credentials,
        header: DatabaseHeader,
        content: DatabaseContent,
    },
    Ver4x {
        credentials: Credentials,
        header: DatabaseHeader,
        content: DatabaseContent,
        inner_header: DatabaseInnerHeader,
    },
}

impl KeePassDatabase {
    pub fn create_ver3x(
        root_name: impl Into<String>,
        meta: Meta,
        credentials: Credentials,
    ) -> Result<Self, FormatError> {
        Ok(Self::Ver3x {
            credentials,
            header: DatabaseHeader::create_ver3x()?,
            content: blank_content(root_name, meta),
        })
    }

    pub fn create_ver4x(
        root_name: impl Into<String>,
        meta: Meta,
        credentials: Credentials,
    ) -> Result<Self, FormatError> {
        Ok(Self::Ver4x {
            credentials,
            header: DatabaseHeader::create_ver4x()?,
            content: blank_content(root_name, meta),
            inner_header: DatabaseInnerHeader::create()?,
        })
    }

    pub const fn credentials(&self) -> &Credentials {
        match self {
            Self::Ver3x { credentials, .. } | Self::Ver4x { credentials, .. } => credentials,
        }
    }

    pub const fn header(&self) -> &DatabaseHeader {
        match self {
            Self::Ver3x { header, .. } | Self::Ver4x { header, .. } => header,
        }
    }

    pub const fn content(&self) -> &DatabaseContent {
        match self {
            Self::Ver3x { content, .. } | Self::Ver4x { content, .. } => content,
        }
    }

    pub fn content_mut(&mut self) -> &mut DatabaseContent {
        match self {
            Self::Ver3x { content, .. } | Self::Ver4x { content, .. } => content,
        }
    }

    pub const fn inner_header(&self) -> Option<&DatabaseInnerHeader> {
        match self {
            Self::Ver3x { .. } => None,
            Self::Ver4x { inner_header, .. } => Some(inner_header),
        }
    }

    pub fn traverse<'a>(&'a self, block: impl FnMut(DatabaseElementRef<'a>)) {
        self.content().group.traverse(block);
    }

    pub fn get_group(
        &self,
        predicate: impl Fn(&Group) -> bool,
    ) -> Option<(Option<&Group>, &Group)> {
        if predicate(&self.content().group) {
            Some((None, &self.content().group))
        } else {
            self.content()
                .group
                .find_child_group(None, predicate)
                .map(|(parent, group)| (Some(parent), group))
        }
    }

    pub fn get_group_by(&self, predicate: impl Fn(&Group) -> bool) -> Option<&Group> {
        self.get_group(predicate).map(|(_, group)| group)
    }

    pub fn get_entry(&self, predicate: impl Fn(&Entry) -> bool) -> Option<(&Group, &Entry)> {
        self.content()
            .group
            .find_child_entry(false, None, predicate)
    }

    pub fn find_entry(&self, predicate: impl Fn(&Entry) -> bool) -> Option<(&Group, &Entry)> {
        self.content()
            .group
            .find_child_entry(true, self.content().meta.recycle_bin_uuid, predicate)
    }

    pub fn get_entry_by(&self, predicate: impl Fn(&Entry) -> bool) -> Option<&Entry> {
        self.get_entry(predicate).map(|(_, entry)| entry)
    }

    pub fn find_entry_by(&self, predicate: impl Fn(&Entry) -> bool) -> Option<&Entry> {
        self.find_entry(predicate).map(|(_, entry)| entry)
    }

    pub fn get_entries(&self, predicate: impl Fn(&Entry) -> bool) -> Vec<(&Group, Vec<&Entry>)> {
        self.content()
            .group
            .find_child_entries(false, None, predicate)
    }

    pub fn find_entries(&self, predicate: impl Fn(&Entry) -> bool) -> Vec<(&Group, Vec<&Entry>)> {
        self.content().group.find_child_entries(
            true,
            self.content().meta.recycle_bin_uuid,
            predicate,
        )
    }

    pub fn measure_key_transform_millis(&self) -> Result<u128, CryptoError> {
        self.measure_key_transform_millis_with(&BaseKdfProvider)
    }

    pub fn measure_key_transform_millis_with(
        &self,
        kdf_provider: &impl KdfProvider,
    ) -> Result<u128, CryptoError> {
        let start = Instant::now();
        let mut transformed_key = self.transformed_key(kdf_provider)?;
        clear(&mut transformed_key);
        Ok(start.elapsed().as_millis())
    }

    fn transformed_key(&self, kdf_provider: &impl KdfProvider) -> Result<Vec<u8>, CryptoError> {
        let mut composite_key = self.credentials().composite_key();
        let result = match self.header() {
            DatabaseHeader::Ver3x {
                transform_rounds,
                transform_seed,
                ..
            } => kdf_provider.transform_key(
                &CryptoKdfParameters::Aes {
                    rounds: *transform_rounds,
                    seed: transform_seed.clone(),
                },
                &composite_key,
            ),
            DatabaseHeader::Ver4x { kdf_parameters, .. } => {
                kdf_provider.transform_key(&kdf_parameters.to_crypto_parameters(), &composite_key)
            }
        };
        clear(&mut composite_key);
        result
    }
}

fn blank_content(root_name: impl Into<String>, meta: Meta) -> DatabaseContent {
    let mut root = Group::new(Uuid::new_v4(), root_name);
    root.enable_auto_type = GroupOverride::Enabled;
    root.enable_searching = GroupOverride::Enabled;
    DatabaseContent::new(meta, root, Vec::new())
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::{KeePassDatabase, MAX_SUPPORTED_VERSION, MIN_SUPPORTED_VERSION};
    use crate::{
        constants::GroupOverride,
        crypto::EncryptedValue,
        database::{Credentials, header::DatabaseHeader},
        model::{Entry, Meta},
    };

    fn credentials() -> Credentials {
        let passphrase = EncryptedValue::from_string("secret").unwrap();
        Credentials::from_passphrase(&passphrase).unwrap()
    }

    #[test]
    fn creates_blank_v3_database() {
        let database =
            KeePassDatabase::create_ver3x("Root", Meta::default(), credentials()).unwrap();

        match database {
            KeePassDatabase::Ver3x {
                header, content, ..
            } => {
                assert!(matches!(header, DatabaseHeader::Ver3x { .. }));
                assert_eq!(content.group.name, "Root");
                assert_eq!(content.group.enable_auto_type, GroupOverride::Enabled);
                assert_eq!(content.group.enable_searching, GroupOverride::Enabled);
                assert!(content.deleted_objects.is_empty());
            }
            KeePassDatabase::Ver4x { .. } => panic!("expected v3 database"),
        }
    }

    #[test]
    fn creates_blank_v4_database() {
        let database =
            KeePassDatabase::create_ver4x("Root", Meta::default(), credentials()).unwrap();

        match database {
            KeePassDatabase::Ver4x {
                header,
                content,
                inner_header,
                ..
            } => {
                assert!(matches!(header, DatabaseHeader::Ver4x { .. }));
                assert_eq!(content.group.name, "Root");
                assert!(inner_header.binaries.is_empty());
            }
            KeePassDatabase::Ver3x { .. } => panic!("expected v4 database"),
        }
    }

    #[test]
    fn exposes_supported_version_bounds() {
        assert_eq!(MIN_SUPPORTED_VERSION, 3);
        assert_eq!(MAX_SUPPORTED_VERSION, 4);
    }

    #[test]
    fn searches_groups_and_entries() {
        let mut database =
            KeePassDatabase::create_ver4x("Root", Meta::default(), credentials()).unwrap();
        let mut child = crate::model::Group::new(Uuid::new_v4(), "Child");
        let entry = Entry::new(Uuid::new_v4());
        let entry_uuid = entry.uuid;
        child.entries.push(entry);
        let child_uuid = child.uuid;
        database.content_mut().group.groups.push(child);

        let (_, group) = database
            .get_group(|group| group.uuid == child_uuid)
            .expect("child group should be found");
        assert_eq!(group.name, "Child");
        assert_eq!(
            database
                .get_entry_by(|entry| entry.uuid == entry_uuid)
                .expect("entry should be found")
                .uuid,
            entry_uuid
        );
        assert_eq!(
            database
                .find_entry_by(|entry| entry.uuid == entry_uuid)
                .expect("entry should be found")
                .uuid,
            entry_uuid
        );
    }
}
