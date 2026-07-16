use indexmap::{IndexMap, IndexSet};

use crate::model::{BinaryData, Entry, Group};

use super::super::KeePassDatabase;

pub type DatabaseBinaries = IndexMap<Vec<u8>, BinaryData>;

impl KeePassDatabase {
    pub fn binaries(&self) -> &DatabaseBinaries {
        match self {
            Self::Ver3x { content, .. } => &content.meta.binaries,
            Self::Ver4x { inner_header, .. } => &inner_header.binaries,
        }
    }

    pub fn binaries_mut(&mut self) -> &mut DatabaseBinaries {
        match self {
            Self::Ver3x { content, .. } => &mut content.meta.binaries,
            Self::Ver4x { inner_header, .. } => &mut inner_header.binaries,
        }
    }

    pub fn modify_binaries(
        &mut self,
        block: impl FnOnce(&mut DatabaseBinaries),
    ) -> &mut KeePassDatabase {
        block(self.binaries_mut());
        self
    }

    pub fn remove_unused_binaries(&mut self) -> &mut KeePassDatabase {
        let referenced = referenced_binary_hashes(&self.content().group);
        self.binaries_mut()
            .retain(|hash, _| referenced.contains(hash));
        self
    }
}

fn referenced_binary_hashes(root: &Group) -> IndexSet<Vec<u8>> {
    let mut referenced = IndexSet::new();
    let mut stack = vec![root];

    while let Some(group) = stack.pop() {
        for entry in &group.entries {
            collect_entry_binaries(entry, &mut referenced);
        }
        for child in &group.groups {
            stack.push(child);
        }
    }

    referenced
}

fn collect_entry_binaries(entry: &Entry, referenced: &mut IndexSet<Vec<u8>>) {
    for binary in &entry.binaries {
        referenced.insert(binary.hash.clone());
    }
    for historical_entry in &entry.history {
        for binary in &historical_entry.binaries {
            referenced.insert(binary.hash.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::{
        crypto::EncryptedValue,
        database::{Credentials, KeePassDatabase},
        model::{BinaryData, BinaryReference, Entry, Meta},
    };

    fn credentials() -> Credentials {
        Credentials::from_passphrase(&EncryptedValue::from_string("1").unwrap()).unwrap()
    }

    #[test]
    fn returns_v3_meta_binaries() {
        let mut database =
            KeePassDatabase::create_ver3x("Root", Meta::default(), credentials()).unwrap();
        let binary = BinaryData::uncompressed(false, b"v3".to_vec());
        let hash = binary.hash().to_vec();
        database
            .content_mut()
            .meta
            .binaries
            .insert(hash.clone(), binary);

        assert!(database.binaries().contains_key(&hash));
    }

    #[test]
    fn returns_v4_inner_header_binaries() {
        let mut database =
            KeePassDatabase::create_ver4x("Root", Meta::default(), credentials()).unwrap();
        let binary = BinaryData::uncompressed(false, b"v4".to_vec());
        let hash = binary.hash().to_vec();
        let KeePassDatabase::Ver4x { inner_header, .. } = &mut database else {
            panic!("expected v4 database");
        };
        inner_header.binaries.insert(hash.clone(), binary);

        assert!(database.binaries().contains_key(&hash));
    }

    #[test]
    fn modifies_binaries_in_place() {
        let mut database =
            KeePassDatabase::create_ver4x("Root", Meta::default(), credentials()).unwrap();
        let binary = BinaryData::uncompressed(false, b"data".to_vec());
        let hash = binary.hash().to_vec();

        database.modify_binaries(|binaries| {
            binaries.insert(hash.clone(), binary);
        });

        assert!(database.binaries().contains_key(&hash));
    }

    #[test]
    fn removes_unused_binaries_but_keeps_entry_and_history_references() {
        let mut database =
            KeePassDatabase::create_ver4x("Root", Meta::default(), credentials()).unwrap();
        let used = BinaryData::uncompressed(false, b"used".to_vec());
        let historical = BinaryData::uncompressed(false, b"history".to_vec());
        let unused = BinaryData::uncompressed(false, b"unused".to_vec());
        let used_hash = used.hash().to_vec();
        let historical_hash = historical.hash().to_vec();
        let unused_hash = unused.hash().to_vec();
        database.modify_binaries(|binaries| {
            binaries.insert(used_hash.clone(), used);
            binaries.insert(historical_hash.clone(), historical);
            binaries.insert(unused_hash.clone(), unused);
        });

        let mut entry = Entry::new(Uuid::new_v4());
        entry
            .binaries
            .push(BinaryReference::new(used_hash.clone(), "used"));
        let mut historical_entry = Entry::new(Uuid::new_v4());
        historical_entry
            .binaries
            .push(BinaryReference::new(historical_hash.clone(), "historical"));
        entry.history.push(historical_entry);
        database.content_mut().group.entries.push(entry);

        database.remove_unused_binaries();

        assert!(database.binaries().contains_key(&used_hash));
        assert!(database.binaries().contains_key(&historical_hash));
        assert!(!database.binaries().contains_key(&unused_hash));
    }
}
