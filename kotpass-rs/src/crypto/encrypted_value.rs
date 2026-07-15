use crate::crypto::{byte_array::sha256, secure_random::secure_random_bytes};
use crate::io::base64::{Base64Error, decode_base64_to_array, encode_base64};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EncryptedValue {
    value: Vec<u8>,
    salt: Vec<u8>,
}

impl EncryptedValue {
    pub fn new(value: Vec<u8>, salt: Vec<u8>) -> Self {
        assert_eq!(value.len(), salt.len(), "value and salt length must match");
        Self { value, salt }
    }

    pub fn from_binary(mut bytes: Vec<u8>) -> Result<Self, rand::rngs::SysError> {
        let salt = secure_random_bytes(bytes.len())?;

        for (byte, salt_byte) in bytes.iter_mut().zip(&salt) {
            *byte ^= salt_byte;
        }

        Ok(Self { value: bytes, salt })
    }

    pub fn from_string(text: impl AsRef<str>) -> Result<Self, rand::rngs::SysError> {
        Self::from_binary(text.as_ref().as_bytes().to_vec())
    }

    pub fn from_base64(base64: &str) -> Result<Self, EncryptedValueError> {
        let bytes = decode_base64_to_array(base64)?;
        Ok(Self::from_binary(bytes)?)
    }

    pub fn byte_len(&self) -> usize {
        self.value.len()
    }

    pub fn get_binary(&self) -> Vec<u8> {
        self.value
            .iter()
            .zip(&self.salt)
            .map(|(value, salt)| value ^ salt)
            .collect()
    }

    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.get_binary()).into_owned()
    }

    pub fn get_hash(&self) -> Vec<u8> {
        sha256(&self.get_binary())
    }

    pub fn set_salt(&mut self, new_salt: Vec<u8>) {
        assert_eq!(
            self.value.len(),
            new_salt.len(),
            "new salt length must match value length"
        );

        for ((value, salt), new_salt_byte) in
            self.value.iter_mut().zip(&mut self.salt).zip(new_salt)
        {
            *value = (*value ^ *salt) ^ new_salt_byte;
            *salt = new_salt_byte;
        }
    }

    pub fn to_base64(&self) -> String {
        encode_base64(&self.get_binary())
    }

    pub fn encrypted_base64(&self) -> String {
        encode_base64(&self.value)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EncryptedValueError {
    #[error(transparent)]
    Base64(#[from] Base64Error),

    #[error(transparent)]
    Random(#[from] rand::rngs::SysError),
}

#[cfg(test)]
mod tests {
    use super::EncryptedValue;

    #[test]
    fn decrypts_xored_value() {
        let value = EncryptedValue::new(vec![0x41 ^ 0x10, 0x42 ^ 0x20], vec![0x10, 0x20]);

        assert_eq!(value.get_binary(), b"AB");
        assert_eq!(value.text(), "AB");
        assert_eq!(value.byte_len(), 2);
    }

    #[test]
    fn changes_salt_without_changing_plaintext() {
        let mut value = EncryptedValue::new(vec![0x41 ^ 0x10], vec![0x10]);

        value.set_salt(vec![0x20]);

        assert_eq!(value.get_binary(), b"A");
    }
}
