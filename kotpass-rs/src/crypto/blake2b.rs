use blake2::{
    Blake2bMac512, Blake2bVar,
    digest::{Mac, Update, VariableOutput},
};

use crate::error::CryptoError;

pub struct Blake2bDigest {
    state: Blake2bState,
    digest_len: usize,
    key: Option<Vec<u8>>,
    salt: Option<Vec<u8>>,
    personalization: Option<Vec<u8>>,
}

enum Blake2bState {
    Variable(Blake2bVar),
    Mac512(Blake2bMac512),
}

impl Blake2bDigest {
    pub fn new(digest_size_bits: usize) -> Result<Self, CryptoError> {
        if digest_size_bits < 8 || digest_size_bits > 512 || !digest_size_bits.is_multiple_of(8) {
            return Err(CryptoError::InvalidDataLength(
                "BLAKE2b digest bit length must be a multiple of 8 and not greater than 512"
                    .to_owned(),
            ));
        }
        Self::with_params(None, digest_size_bits / 8, None, None)
    }

    pub fn with_key(key: Option<&[u8]>) -> Result<Self, CryptoError> {
        Self::with_params(key, 64, None, None)
    }

    pub fn with_params(
        key: Option<&[u8]>,
        digest_len: usize,
        salt: Option<&[u8]>,
        personalization: Option<&[u8]>,
    ) -> Result<Self, CryptoError> {
        if !(1..=64).contains(&digest_len) {
            return Err(CryptoError::InvalidDataLength(
                "Invalid digest length (required: 1-64).".to_owned(),
            ));
        }
        if let Some(key) = key
            && key.len() > 64
        {
            return Err(CryptoError::InvalidKey(
                "Keys > 64 are not supported.".to_owned(),
            ));
        }
        if let Some(salt) = salt
            && salt.len() != 16
        {
            return Err(CryptoError::InvalidDataLength(
                "Salt length must be exactly 16 bytes.".to_owned(),
            ));
        }
        if let Some(personalization) = personalization
            && personalization.len() != 16
        {
            return Err(CryptoError::InvalidDataLength(
                "Personalization length must be exactly 16 bytes.".to_owned(),
            ));
        }

        let mut digest = Self {
            state: Blake2bState::Variable(Blake2bVar::new(64).expect("valid output length")),
            digest_len,
            key: key.map(ToOwned::to_owned),
            salt: salt.map(ToOwned::to_owned),
            personalization: personalization.map(ToOwned::to_owned),
        };
        digest.state = digest.make_state()?;
        Ok(digest)
    }

    pub fn update(&mut self, message: &[u8], offset: usize, len: usize) -> Result<(), CryptoError> {
        let end = offset
            .checked_add(len)
            .ok_or_else(|| CryptoError::InvalidDataLength("Input buffer too short".to_owned()))?;
        if end > message.len() {
            return Err(CryptoError::InvalidDataLength(
                "Input buffer too short".to_owned(),
            ));
        }

        let slice = &message[offset..end];
        match &mut self.state {
            Blake2bState::Variable(hasher) => Update::update(hasher, slice),
            Blake2bState::Mac512(hasher) => Update::update(hasher, slice),
        }
        Ok(())
    }

    pub fn update_byte(&mut self, byte: u8) {
        match &mut self.state {
            Blake2bState::Variable(hasher) => Update::update(hasher, &[byte]),
            Blake2bState::Mac512(hasher) => Update::update(hasher, &[byte]),
        }
    }

    pub fn do_final(&mut self, out: &mut [u8], out_offset: usize) -> Result<usize, CryptoError> {
        if out_offset
            .checked_add(self.digest_len)
            .is_none_or(|end| end > out.len())
        {
            return Err(CryptoError::InvalidDataLength(
                "Output buffer too short".to_owned(),
            ));
        }

        let replacement = self.make_state()?;
        let state = std::mem::replace(&mut self.state, replacement);
        let output = &mut out[out_offset..out_offset + self.digest_len];

        match state {
            Blake2bState::Variable(hasher) => hasher.finalize_variable(output).map_err(|_| {
                CryptoError::InvalidDataLength("Output buffer too short".to_owned())
            })?,
            Blake2bState::Mac512(hasher) => {
                let tag = hasher.finalize().into_bytes();
                output.copy_from_slice(&tag[..self.digest_len]);
            }
        }
        Ok(self.digest_len)
    }

    pub fn digest_len(&self) -> usize {
        self.digest_len
    }

    pub fn reset(&mut self) -> Result<(), CryptoError> {
        self.state = self.make_state()?;
        Ok(())
    }

    pub fn clear_key(&mut self) {
        if let Some(key) = &mut self.key {
            key.fill(0);
        }
    }

    pub fn clear_salt(&mut self) {
        if let Some(salt) = &mut self.salt {
            salt.fill(0);
        }
    }

    fn make_state(&self) -> Result<Blake2bState, CryptoError> {
        let key = self.key.as_deref();
        let salt = self.salt.as_deref();
        let personalization = self.personalization.as_deref();

        if key.is_none() && salt.is_none() && personalization.is_none() {
            return Ok(Blake2bState::Variable(
                Blake2bVar::new(self.digest_len).map_err(|_| {
                    CryptoError::InvalidDataLength(
                        "Invalid digest length (required: 1-64).".to_owned(),
                    )
                })?,
            ));
        }

        if self.digest_len != 64 {
            return Err(CryptoError::AlgorithmUnavailable(
                "Keyed, salted, or personalized BLAKE2b currently requires a 64-byte digest"
                    .to_owned(),
            ));
        }

        Ok(Blake2bState::Mac512(
            Blake2bMac512::new_with_salt_and_personal(
                key.unwrap_or(&[]),
                salt.unwrap_or(&[]),
                personalization.unwrap_or(&[]),
            )
            .map_err(|_| CryptoError::InvalidDataLength("Invalid BLAKE2b parameters".to_owned()))?,
        ))
    }
}

impl Default for Blake2bDigest {
    fn default() -> Self {
        Self::new(512).expect("default BLAKE2b length is valid")
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::Blake2bDigest;

    #[test]
    fn hashes_empty_input_with_blake2b_512() {
        let mut digest = Blake2bDigest::new(512).unwrap();
        let mut out = [0; 64];

        digest.do_final(&mut out, 0).unwrap();

        assert_eq!(
            hex::encode(out),
            "786a02f742015903c6c6fd852552d272912f4740e15847618a86e217f71f5419d25e1031afee585313896444934eb04b903a685b1448b755d56f701afe9be2ce"
        );
    }

    #[test]
    fn hashes_variable_length_output() {
        let mut digest = Blake2bDigest::new(256).unwrap();
        digest.update(b"abc", 0, 3).unwrap();
        let mut out = [0; 32];

        assert_eq!(digest.do_final(&mut out, 0), Ok(32));

        assert_eq!(
            hex::encode(out),
            "bddd813c634239723171ef3fee98579b94964e3bb1cb3e427262c8c068d52319"
        );
    }

    #[test]
    fn hashes_keyed_kotlin_vector() {
        let input: Vec<u8> = (0..=0xe8).collect();
        let key: Vec<u8> = (0..=0x3f).collect();
        let mut digest = Blake2bDigest::with_params(Some(&key), 64, None, None).unwrap();
        let mut out = [0; 64];

        digest.update(&input, 0, input.len()).unwrap();
        digest.do_final(&mut out, 0).unwrap();

        assert_eq!(
            hex::encode(out),
            "d4762cd4599876ca75b2b8fe249944dbd27ace741fdab93616cbc6e425460feb51d4e7adcc38180e7fc47c89024a7f56191adb878dfde4ead62223f5a2610efe"
        );
    }
}
