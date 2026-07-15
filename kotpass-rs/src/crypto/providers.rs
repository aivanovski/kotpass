use aes::Aes256;
use cipher::{BlockModeDecrypt, BlockModeEncrypt, KeyIvInit, block_padding::Pkcs7};
use uuid::Uuid;

use crate::{
    constants::CrsAlgorithm,
    crypto::{
        ChaCha7539Engine, PaddedBufferedBlockCipher, Pkcs7Padding, Salsa20Engine, TwofishEngine,
        byte_array::{sha256, sha512},
        cipher::BlockCipherMode,
    },
    error::{CryptoError, FormatError},
};

pub trait CipherProvider {
    fn uuid(&self) -> Uuid;
    fn iv_len(&self) -> usize;
    fn encrypt(&self, key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, CryptoError>;
    fn decrypt(&self, key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, CryptoError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BaseCipher {
    Aes,
    ChaCha20,
}

impl BaseCipher {
    pub const ALL: [Self; 2] = [Self::Aes, Self::ChaCha20];
}

impl CipherProvider for BaseCipher {
    fn uuid(&self) -> Uuid {
        match self {
            Self::Aes => Uuid::parse_str("31c1f2e6-bf71-4350-be58-05216afc5aff")
                .expect("valid AES cipher UUID"),
            Self::ChaCha20 => Uuid::parse_str("d6038a2b-8b6f-4cb5-a524-339a31dbb59a")
                .expect("valid ChaCha20 cipher UUID"),
        }
    }

    fn iv_len(&self) -> usize {
        match self {
            Self::Aes => 16,
            Self::ChaCha20 => 12,
        }
    }

    fn encrypt(&self, key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        match self {
            Self::Aes => aes_cbc_encrypt(key, iv, data),
            Self::ChaCha20 => ChaCha7539Engine::new(key, iv)?.process_bytes(data),
        }
    }

    fn decrypt(&self, key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        match self {
            Self::Aes => aes_cbc_decrypt(key, iv, data),
            Self::ChaCha20 => ChaCha7539Engine::new(key, iv)?.process_bytes(data),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct TwofishCipher;

impl CipherProvider for TwofishCipher {
    fn uuid(&self) -> Uuid {
        Uuid::parse_str("ad68f29f-576f-4bb9-a36a-d47af965346c").expect("valid Twofish cipher UUID")
    }

    fn iv_len(&self) -> usize {
        16
    }

    fn encrypt(&self, key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let mut cipher = PaddedBufferedBlockCipher::new(
            TwofishEngine::new(),
            BlockCipherMode::cbc(iv),
            Pkcs7Padding,
        );
        cipher.init(true, key)?;
        cipher.process_bytes_to_vec(data)
    }

    fn decrypt(&self, key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let mut cipher = PaddedBufferedBlockCipher::new(
            TwofishEngine::new(),
            BlockCipherMode::cbc(iv),
            Pkcs7Padding,
        );
        cipher.init(false, key)?;
        cipher.process_bytes_to_vec(data)
    }
}

pub enum EncryptionSaltGenerator {
    Salsa20(Salsa20Engine),
    ChaCha20(ChaCha7539Engine),
}

impl EncryptionSaltGenerator {
    pub fn salsa20(key: &[u8]) -> Result<Self, CryptoError> {
        let nonce = [0xe8, 0x30, 0x09, 0x4b, 0x97, 0x20, 0x5d, 0x2a];
        Ok(Self::Salsa20(Salsa20Engine::new(&sha256(key), &nonce)?))
    }

    pub fn chacha20(key: &[u8]) -> Result<Self, CryptoError> {
        let hash = sha512(key);
        Ok(Self::ChaCha20(ChaCha7539Engine::new(
            &hash[..32],
            &hash[32..44],
        )?))
    }

    pub fn create(id: CrsAlgorithm, key: &[u8]) -> Result<Self, FormatError> {
        match id {
            CrsAlgorithm::Salsa20 => {
                Self::salsa20(key).map_err(|err| FormatError::InvalidHeader(err.to_string()))
            }
            CrsAlgorithm::ChaCha20 => {
                Self::chacha20(key).map_err(|err| FormatError::InvalidHeader(err.to_string()))
            }
            _ => Err(FormatError::InvalidHeader(
                "Unsupported inner random stream cipher.".to_owned(),
            )),
        }
    }

    pub fn get_salt(&mut self, length: usize) -> Result<Vec<u8>, CryptoError> {
        match self {
            Self::Salsa20(engine) => engine.get_bytes(length),
            Self::ChaCha20(engine) => engine.get_bytes(length),
        }
    }

    pub fn process_bytes(&mut self, input: &[u8]) -> Result<Vec<u8>, CryptoError> {
        match self {
            Self::Salsa20(engine) => engine.process_bytes(input),
            Self::ChaCha20(engine) => engine.process_bytes(input),
        }
    }
}

fn aes_cbc_encrypt(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    cbc::Encryptor::<Aes256>::new_from_slices(key, iv)
        .map_err(|_| CryptoError::InvalidKey("Wrong key used for encryption.".to_owned()))
        .map(|cipher| cipher.encrypt_padded_vec::<Pkcs7>(data))
}

fn aes_cbc_decrypt(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    cbc::Decryptor::<Aes256>::new_from_slices(key, iv)
        .map_err(|_| CryptoError::InvalidKey("Wrong key used for decryption.".to_owned()))?
        .decrypt_padded_vec::<Pkcs7>(data)
        .map_err(|_| CryptoError::InvalidKey("Wrong key used for decryption.".to_owned()))
}

#[cfg(test)]
mod tests {
    use crate::{
        crypto::{BaseCipher, CipherProvider, EncryptionSaltGenerator, TwofishCipher},
        io::base64::decode_base64_to_array,
    };

    #[test]
    fn salsa20_salt_generator_matches_kotlin_sequence() {
        let mut generator = EncryptionSaltGenerator::salsa20(&[1, 2, 3]).unwrap();

        assert_eq!(generator.get_salt(0).unwrap().len(), 0);
        assert_eq!(
            generator.get_salt(10).unwrap(),
            decode_base64_to_array("q1l4McuyQYDcDg==").unwrap()
        );
        assert_eq!(
            generator.get_salt(10).unwrap(),
            decode_base64_to_array("LJTKXBjqlTS8cg==").unwrap()
        );
        assert_eq!(
            generator.get_salt(20).unwrap(),
            decode_base64_to_array("jKVBKKNUnieRr47Wxh0YTKn82Pw=").unwrap()
        );
    }

    #[test]
    fn chacha20_salt_generator_matches_kotlin_sequence() {
        let mut generator = EncryptionSaltGenerator::chacha20(&[1, 2, 3]).unwrap();

        assert_eq!(generator.get_salt(0).unwrap().len(), 0);
        assert_eq!(
            generator.get_salt(10).unwrap(),
            decode_base64_to_array("iUIv7m2BJN2ubQ==").unwrap()
        );
        assert_eq!(
            generator.get_salt(10).unwrap(),
            decode_base64_to_array("BILRgZKxaxbRzg==").unwrap()
        );
        assert_eq!(
            generator.get_salt(20).unwrap(),
            decode_base64_to_array("KUeBUGjNBYhAoJstSqnMXQwuD6E=").unwrap()
        );
    }

    #[test]
    fn base_ciphers_round_trip() {
        let key = [1; 32];
        let aes_iv = [2; 16];
        let chacha_iv = [3; 12];
        let plain = b"database payload";

        let encrypted = BaseCipher::Aes.encrypt(&key, &aes_iv, plain).unwrap();
        assert_eq!(
            BaseCipher::Aes.decrypt(&key, &aes_iv, &encrypted).unwrap(),
            plain
        );

        let encrypted = BaseCipher::ChaCha20
            .encrypt(&key, &chacha_iv, plain)
            .unwrap();
        assert_eq!(
            BaseCipher::ChaCha20
                .decrypt(&key, &chacha_iv, &encrypted)
                .unwrap(),
            plain
        );
    }

    #[test]
    fn twofish_provider_round_trips() {
        let key = [4; 32];
        let iv = [5; 16];
        let plain = b"twofish database payload";

        let encrypted = TwofishCipher.encrypt(&key, &iv, plain).unwrap();

        assert_eq!(TwofishCipher.decrypt(&key, &iv, &encrypted).unwrap(), plain);
    }
}
