use indexmap::IndexMap;

use crate::{
    constants::kdf_const::keys,
    crypto::{
        Argon2Variant as CryptoArgon2Variant, Argon2Version, KdfParameters as CryptoKdfParameters,
    },
    error::FormatError,
};

use super::{VariantDictionary, VariantItem, VariantItems};

const AES_UUID: [u8; 16] = [
    0xC9, 0xD9, 0xF3, 0x9A, 0x62, 0x8A, 0x44, 0x60, 0xBF, 0x74, 0x0D, 0x08, 0xC1, 0x8A, 0x4F, 0xEA,
];

const ARGON2D_UUID: [u8; 16] = [
    0xEF, 0x63, 0x6D, 0xDF, 0x8C, 0x29, 0x44, 0x4B, 0x91, 0xF7, 0xA9, 0xA4, 0x03, 0xE3, 0x0A, 0x0C,
];

const ARGON2ID_UUID: [u8; 16] = [
    0x9E, 0x29, 0x8B, 0x19, 0x56, 0xDB, 0x47, 0x73, 0xB2, 0x3D, 0xFC, 0x3E, 0xC6, 0xF0, 0xA1, 0xE6,
];

/// Describes key-derivation function parameters stored in KDBX headers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KdfParameters {
    Aes {
        rounds: u64,
        seed: Vec<u8>,
    },
    Argon2 {
        variant: KdfArgon2Variant,
        salt: Vec<u8>,
        parallelism: u32,
        memory: u64,
        iterations: u64,
        version: u32,
        secret_key: Option<Vec<u8>>,
        associated_data: Option<Vec<u8>>,
    },
}

impl KdfParameters {
    pub const AES_UUID: &'static [u8; 16] = &AES_UUID;
    pub const ARGON2D_UUID: &'static [u8; 16] = &ARGON2D_UUID;
    pub const ARGON2ID_UUID: &'static [u8; 16] = &ARGON2ID_UUID;

    pub fn argon2_default(salt: impl Into<Vec<u8>>) -> Self {
        Self::Argon2 {
            variant: KdfArgon2Variant::Argon2d,
            salt: salt.into(),
            parallelism: 2,
            memory: 32 * 1024 * 1024,
            iterations: 8,
            version: Argon2Version::Ver13.id(),
            secret_key: None,
            associated_data: None,
        }
    }

    pub const fn uuid(&self) -> &'static [u8; 16] {
        match self {
            Self::Aes { .. } => Self::AES_UUID,
            Self::Argon2 { variant, .. } => variant.uuid(),
        }
    }

    pub fn read_from(data: &[u8]) -> Result<Self, FormatError> {
        let items = VariantDictionary::read_from(data)?;
        let uuid = required_bytes(&items, keys::UUID, "No KDF UUID found.")?;

        if uuid == Self::AES_UUID {
            Ok(Self::Aes {
                rounds: required_u64(&items, keys::ROUNDS, "No KDF rounds found.")?,
                seed: required_bytes(&items, keys::SALT_OR_SEED, "No KDF seed found.")?,
            })
        } else if let Some(variant) = KdfArgon2Variant::from_uuid(&uuid) {
            Ok(Self::Argon2 {
                variant,
                salt: required_bytes(&items, keys::SALT_OR_SEED, "No KDF salt found.")?,
                parallelism: required_u32(&items, keys::PARALLELISM, "No KDF parallelism found.")?,
                memory: required_u64(&items, keys::MEMORY, "No KDF memory found.")?,
                iterations: required_u64(&items, keys::ITERATIONS, "No KDF iterations found.")?,
                version: required_u32(&items, keys::VERSION, "No KDF version found.")?,
                secret_key: optional_bytes(&items, keys::SECRET_KEY),
                associated_data: optional_bytes(&items, keys::ASSOC_DATA),
            })
        } else {
            Err(FormatError::InvalidHeader("Unknown KDF UUID.".to_owned()))
        }
    }

    pub fn write_to_bytes(&self) -> Vec<u8> {
        let mut items = IndexMap::new();

        match self {
            Self::Aes { rounds, seed } => {
                items.insert(
                    keys::UUID.to_owned(),
                    VariantItem::Bytes(Self::AES_UUID.to_vec()),
                );
                items.insert(keys::ROUNDS.to_owned(), VariantItem::UInt64(*rounds));
                items.insert(
                    keys::SALT_OR_SEED.to_owned(),
                    VariantItem::Bytes(seed.clone()),
                );
            }
            Self::Argon2 {
                variant,
                salt,
                parallelism,
                memory,
                iterations,
                version,
                secret_key,
                associated_data,
            } => {
                items.insert(
                    keys::UUID.to_owned(),
                    VariantItem::Bytes(variant.uuid().to_vec()),
                );
                items.insert(
                    keys::SALT_OR_SEED.to_owned(),
                    VariantItem::Bytes(salt.clone()),
                );
                items.insert(
                    keys::PARALLELISM.to_owned(),
                    VariantItem::UInt32(*parallelism),
                );
                items.insert(keys::MEMORY.to_owned(), VariantItem::UInt64(*memory));
                items.insert(
                    keys::ITERATIONS.to_owned(),
                    VariantItem::UInt64(*iterations),
                );
                items.insert(keys::VERSION.to_owned(), VariantItem::UInt32(*version));

                if let Some(secret_key) = secret_key {
                    items.insert(
                        keys::SECRET_KEY.to_owned(),
                        VariantItem::Bytes(secret_key.clone()),
                    );
                }
                if let Some(associated_data) = associated_data {
                    items.insert(
                        keys::ASSOC_DATA.to_owned(),
                        VariantItem::Bytes(associated_data.clone()),
                    );
                }
            }
        }

        VariantDictionary::write_to_bytes(&items)
    }

    pub fn to_crypto_parameters(&self) -> CryptoKdfParameters {
        match self {
            Self::Aes { rounds, seed } => CryptoKdfParameters::Aes {
                rounds: *rounds,
                seed: seed.clone(),
            },
            Self::Argon2 {
                variant,
                salt,
                parallelism,
                memory,
                iterations,
                version,
                secret_key,
                associated_data,
            } => CryptoKdfParameters::Argon2 {
                variant: variant.crypto_variant(),
                version: Argon2Version::from_id(*version),
                salt: salt.clone(),
                secret_key: secret_key.clone(),
                associated_data: associated_data.clone(),
                iterations: *iterations,
                parallelism: *parallelism,
                memory: *memory,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KdfArgon2Variant {
    Argon2d,
    Argon2id,
}

impl KdfArgon2Variant {
    pub const fn uuid(self) -> &'static [u8; 16] {
        match self {
            Self::Argon2d => KdfParameters::ARGON2D_UUID,
            Self::Argon2id => KdfParameters::ARGON2ID_UUID,
        }
    }

    pub fn from_uuid(uuid: &[u8]) -> Option<Self> {
        if uuid == ARGON2D_UUID {
            Some(Self::Argon2d)
        } else if uuid == ARGON2ID_UUID {
            Some(Self::Argon2id)
        } else {
            None
        }
    }

    pub const fn crypto_variant(self) -> CryptoArgon2Variant {
        match self {
            Self::Argon2d => CryptoArgon2Variant::Argon2d,
            Self::Argon2id => CryptoArgon2Variant::Argon2id,
        }
    }
}

impl From<&KdfParameters> for CryptoKdfParameters {
    fn from(parameters: &KdfParameters) -> Self {
        parameters.to_crypto_parameters()
    }
}

fn required_bytes(items: &VariantItems, key: &str, message: &str) -> Result<Vec<u8>, FormatError> {
    match items.get(key) {
        Some(VariantItem::Bytes(value)) => Ok(value.clone()),
        _ => Err(FormatError::InvalidHeader(message.to_owned())),
    }
}

fn optional_bytes(items: &VariantItems, key: &str) -> Option<Vec<u8>> {
    match items.get(key) {
        Some(VariantItem::Bytes(value)) => Some(value.clone()),
        _ => None,
    }
}

fn required_u32(items: &VariantItems, key: &str, message: &str) -> Result<u32, FormatError> {
    match items.get(key) {
        Some(VariantItem::UInt32(value)) => Ok(*value),
        _ => Err(FormatError::InvalidHeader(message.to_owned())),
    }
}

fn required_u64(items: &VariantItems, key: &str, message: &str) -> Result<u64, FormatError> {
    match items.get(key) {
        Some(VariantItem::UInt64(value)) => Ok(*value),
        _ => Err(FormatError::InvalidHeader(message.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use super::{KdfArgon2Variant, KdfParameters};
    use crate::{
        constants::kdf_const::keys,
        crypto::{
            Argon2Variant as CryptoArgon2Variant, Argon2Version,
            KdfParameters as CryptoKdfParameters,
        },
        database::header::{VariantDictionary, VariantItem},
        io::base16::decode_hex_to_array,
    };

    #[test]
    fn reads_kotlin_argon2_fixture() {
        let bytes = include_bytes!("../../../../kotpass/kotpass/src/test/resources/kdf_params");
        let parameters = KdfParameters::read_from(bytes).unwrap();

        match &parameters {
            KdfParameters::Argon2 {
                variant,
                salt,
                parallelism,
                memory,
                iterations,
                version,
                secret_key,
                associated_data,
            } => {
                assert_eq!(*variant, KdfArgon2Variant::Argon2d);
                assert_eq!(
                    parameters.uuid().as_slice(),
                    decode_hex_to_array("ef636ddf8c29444b91f7a9a403e30a0c").unwrap()
                );
                assert!(!salt.is_empty());
                assert!(*parallelism > 0);
                assert!(*memory > 0);
                assert!(*iterations > 0);
                assert_eq!(*version, Argon2Version::Ver13.id());
                assert_eq!(secret_key, &None);
                assert_eq!(associated_data, &None);
            }
            KdfParameters::Aes { .. } => panic!("expected argon2 parameters"),
        }

        let round_tripped = KdfParameters::read_from(&parameters.write_to_bytes()).unwrap();
        assert_eq!(round_tripped, parameters);
    }

    #[test]
    fn writes_and_reads_aes_parameters() {
        let parameters = KdfParameters::Aes {
            rounds: 6000,
            seed: vec![1; 32],
        };

        let decoded = KdfParameters::read_from(&parameters.write_to_bytes()).unwrap();

        assert_eq!(decoded, parameters);
        assert_eq!(decoded.uuid(), KdfParameters::AES_UUID);
    }

    #[test]
    fn writes_and_reads_argon2_parameters_with_optional_fields() {
        let parameters = KdfParameters::Argon2 {
            variant: KdfArgon2Variant::Argon2id,
            salt: vec![2; 16],
            parallelism: 4,
            memory: 64 * 1024 * 1024,
            iterations: 3,
            version: Argon2Version::Ver13.id(),
            secret_key: Some(vec![3; 8]),
            associated_data: Some(vec![4; 12]),
        };

        let decoded = KdfParameters::read_from(&parameters.write_to_bytes()).unwrap();

        assert_eq!(decoded, parameters);
        assert_eq!(decoded.uuid(), KdfParameters::ARGON2ID_UUID);
    }

    #[test]
    fn argon2_default_matches_kotlin_defaults() {
        let parameters = KdfParameters::argon2_default(vec![9; 16]);

        assert_eq!(
            parameters,
            KdfParameters::Argon2 {
                variant: KdfArgon2Variant::Argon2d,
                salt: vec![9; 16],
                parallelism: 2,
                memory: 32 * 1024 * 1024,
                iterations: 8,
                version: Argon2Version::Ver13.id(),
                secret_key: None,
                associated_data: None,
            }
        );
    }

    #[test]
    fn converts_to_crypto_kdf_parameters() {
        let parameters = KdfParameters::Argon2 {
            variant: KdfArgon2Variant::Argon2id,
            salt: vec![2; 16],
            parallelism: 4,
            memory: 64 * 1024 * 1024,
            iterations: 3,
            version: Argon2Version::Ver13.id(),
            secret_key: Some(vec![3; 8]),
            associated_data: Some(vec![4; 12]),
        };

        assert_eq!(
            parameters.to_crypto_parameters(),
            CryptoKdfParameters::Argon2 {
                variant: CryptoArgon2Variant::Argon2id,
                version: Argon2Version::Ver13,
                salt: vec![2; 16],
                secret_key: Some(vec![3; 8]),
                associated_data: Some(vec![4; 12]),
                iterations: 3,
                parallelism: 4,
                memory: 64 * 1024 * 1024,
            }
        );
    }

    #[test]
    fn rejects_missing_uuid() {
        let items = IndexMap::new();
        let error = KdfParameters::read_from(&VariantDictionary::write_to_bytes(&items))
            .expect_err("uuid should be required");

        assert_eq!(error.to_string(), "No KDF UUID found.");
    }

    #[test]
    fn rejects_unknown_uuid() {
        let mut items = IndexMap::new();
        items.insert(keys::UUID.to_owned(), VariantItem::Bytes(vec![0; 16]));

        let error = KdfParameters::read_from(&VariantDictionary::write_to_bytes(&items))
            .expect_err("uuid should be rejected");

        assert_eq!(error.to_string(), "Unknown KDF UUID.");
    }

    #[test]
    fn rejects_missing_argon2_fields_with_kotlin_messages() {
        let mut items = IndexMap::new();
        items.insert(
            keys::UUID.to_owned(),
            VariantItem::Bytes(KdfParameters::ARGON2D_UUID.to_vec()),
        );

        let error = KdfParameters::read_from(&VariantDictionary::write_to_bytes(&items))
            .expect_err("salt should be required");

        assert_eq!(error.to_string(), "No KDF salt found.");
    }
}
