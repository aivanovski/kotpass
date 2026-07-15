use aes::Aes256;
use cipher::{Block, BlockCipherEncrypt, KeyInit};

use crate::{
    crypto::{
        Argon2Engine, Argon2Variant, Argon2Version,
        byte_array::{clear, sha256, sha512},
    },
    error::CryptoError,
};

pub struct AesKdf;

impl AesKdf {
    pub fn transform_key(key: &[u8], seed: &[u8], rounds: u64) -> Result<Vec<u8>, CryptoError> {
        if key.len() != 32 {
            return Err(CryptoError::InvalidKey(
                "AES KDF key must be 32 bytes".to_owned(),
            ));
        }

        let cipher = Aes256::new_from_slice(seed).map_err(|_| {
            CryptoError::InvalidKey("Wrong KDF seed used for decryption.".to_owned())
        })?;
        let mut bytes = key.to_vec();

        for _ in 0..rounds {
            let mut left = Block::<Aes256>::default();
            left.copy_from_slice(&bytes[..16]);
            cipher.encrypt_block(&mut left);
            bytes[..16].copy_from_slice(&left);

            let mut right = Block::<Aes256>::default();
            right.copy_from_slice(&bytes[16..32]);
            cipher.encrypt_block(&mut right);
            bytes[16..32].copy_from_slice(&right);
        }

        let result = sha256(&bytes);
        clear(&mut bytes);
        Ok(result)
    }
}

pub struct Argon2Kdf;

impl Argon2Kdf {
    pub fn transform_key(
        variant: Argon2Variant,
        version: Argon2Version,
        password: &[u8],
        secret_key: Option<Vec<u8>>,
        additional: Option<Vec<u8>>,
        salt: Vec<u8>,
        iterations: u64,
        parallelism: u32,
        memory_bytes: u64,
    ) -> Result<Vec<u8>, CryptoError> {
        let memory_kib = u32::try_from(memory_bytes / 1024)
            .map_err(|_| CryptoError::InvalidKey("Argon2 memory is too large".to_owned()))?;
        let iterations = u32::try_from(iterations)
            .map_err(|_| CryptoError::InvalidKey("Argon2 iterations are too large".to_owned()))?;

        Argon2Engine::new(
            variant,
            version,
            salt,
            secret_key,
            additional,
            iterations,
            parallelism,
            memory_kib,
        )
        .generate_vec(password, 32)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KdfParameters {
    Aes {
        rounds: u64,
        seed: Vec<u8>,
    },
    Argon2 {
        variant: Argon2Variant,
        version: Argon2Version,
        salt: Vec<u8>,
        secret_key: Option<Vec<u8>>,
        associated_data: Option<Vec<u8>>,
        iterations: u64,
        parallelism: u32,
        memory: u64,
    },
}

pub trait KdfProvider {
    fn transform_key(
        &self,
        kdf_parameters: &KdfParameters,
        composite_key: &[u8],
    ) -> Result<Vec<u8>, CryptoError>;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct BaseKdfProvider;

impl KdfProvider for BaseKdfProvider {
    fn transform_key(
        &self,
        kdf_parameters: &KdfParameters,
        composite_key: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        match kdf_parameters {
            KdfParameters::Aes { rounds, seed } => {
                AesKdf::transform_key(composite_key, seed, *rounds)
            }
            KdfParameters::Argon2 {
                variant,
                version,
                salt,
                secret_key,
                associated_data,
                iterations,
                parallelism,
                memory,
            } => Argon2Kdf::transform_key(
                *variant,
                *version,
                composite_key,
                secret_key.clone(),
                associated_data.clone(),
                salt.clone(),
                *iterations,
                *parallelism,
                *memory,
            ),
        }
    }
}

pub struct KeyTransform;

impl KeyTransform {
    pub fn composite_key(passphrase: Option<&[u8]>, key: Option<&[u8]>) -> Vec<u8> {
        let mut composite = Vec::new();
        if let Some(passphrase) = passphrase {
            composite.extend_from_slice(passphrase);
        }
        if let Some(key) = key {
            composite.extend_from_slice(key);
        }

        let result = sha256(&composite);
        clear(&mut composite);
        result
    }

    pub fn master_key(master_seed: &[u8], transformed_key: &[u8]) -> Vec<u8> {
        let mut combined = Vec::with_capacity(master_seed.len() + transformed_key.len());
        combined.extend_from_slice(master_seed);
        combined.extend_from_slice(transformed_key);
        let result = sha256(&combined);
        clear(&mut combined);
        result
    }

    pub fn hmac_key(master_seed: &[u8], transformed_key: &[u8]) -> Vec<u8> {
        let mut combined = Vec::with_capacity(master_seed.len() + transformed_key.len() + 1);
        combined.extend_from_slice(master_seed);
        combined.extend_from_slice(transformed_key);
        combined.push(0x01);

        let combined_hash = sha512(&combined);
        let mut prefixed = vec![0xff; 8];
        prefixed.extend_from_slice(&combined_hash);

        let result = sha512(&prefixed);
        clear(&mut combined);
        clear(&mut prefixed);
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::{AesKdf, KeyTransform};

    #[test]
    fn aes_kdf_matches_kotlin_vector_one() {
        let result = AesKdf::transform_key(
            b"8ee89711330c1ccf39a2e65ad12bbd7d",
            b"a25ca73c7189e2a2ca5acf2088b57e28",
            6000,
        )
        .unwrap();

        assert_eq!(
            result,
            vec![
                0x2f, 0xfa, 0x2c, 0x11, 0xeb, 0x4a, 0xcc, 0xe3, 0x45, 0xd9, 0x9b, 0x53, 0xab, 0x4b,
                0x71, 0x9c, 0xbe, 0x3a, 0x8c, 0x80, 0x99, 0x6f, 0xc7, 0xae, 0xb5, 0xde, 0x76, 0xef,
                0x3e, 0x4c, 0x2d, 0x57,
            ]
        );
    }

    #[test]
    fn aes_kdf_matches_kotlin_vector_two() {
        let passphrase_hash = crate::crypto::byte_array::sha256(b"secret");
        let composite_key = KeyTransform::composite_key(Some(&passphrase_hash), None);
        let seed = [1; 32];

        let result = AesKdf::transform_key(&composite_key, &seed, 10).unwrap();

        assert_eq!(
            result,
            vec![
                208, 2, 238, 193, 16, 181, 39, 109, 254, 40, 67, 20, 154, 21, 202, 174, 234, 11,
                183, 136, 22, 136, 58, 102, 52, 40, 129, 244, 194, 223, 211, 108,
            ]
        );
    }
}
