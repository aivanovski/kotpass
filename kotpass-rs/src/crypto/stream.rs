use chacha20::{ChaCha20, ChaCha20Legacy, Key as ChaChaKey, LegacyNonce, Nonce as ChaChaNonce};
use cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};
use salsa20::{Key as SalsaKey, Nonce as SalsaNonce, Salsa20};

use crate::error::CryptoError;

pub fn chacha_core(rounds: usize, input: &[u32; 16]) -> [u32; 16] {
    assert!(rounds % 2 == 0, "Number of rounds must be even");

    let mut x = *input;
    for _ in (0..rounds).step_by(2) {
        quarter_round(&mut x, 0, 4, 8, 12);
        quarter_round(&mut x, 1, 5, 9, 13);
        quarter_round(&mut x, 2, 6, 10, 14);
        quarter_round(&mut x, 3, 7, 11, 15);
        quarter_round(&mut x, 0, 5, 10, 15);
        quarter_round(&mut x, 1, 6, 11, 12);
        quarter_round(&mut x, 2, 7, 8, 13);
        quarter_round(&mut x, 3, 4, 9, 14);
    }

    for (word, input_word) in x.iter_mut().zip(input) {
        *word = word.wrapping_add(*input_word);
    }
    x
}

fn quarter_round(x: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
    x[a] = x[a].wrapping_add(x[b]);
    x[d] = (x[d] ^ x[a]).rotate_left(16);
    x[c] = x[c].wrapping_add(x[d]);
    x[b] = (x[b] ^ x[c]).rotate_left(12);
    x[a] = x[a].wrapping_add(x[b]);
    x[d] = (x[d] ^ x[a]).rotate_left(8);
    x[c] = x[c].wrapping_add(x[d]);
    x[b] = (x[b] ^ x[c]).rotate_left(7);
}

pub struct Salsa20Engine {
    cipher: Salsa20,
}

impl Salsa20Engine {
    pub const ALGORITHM_NAME: &'static str = "Salsa20";

    pub fn new(key: &[u8], iv: &[u8]) -> Result<Self, CryptoError> {
        let key = salsa_key(key)?;
        let iv = salsa_iv(iv)?;
        Ok(Self {
            cipher: Salsa20::new(&key, &iv),
        })
    }

    pub fn get_bytes(&mut self, number_of_bytes: usize) -> Result<Vec<u8>, CryptoError> {
        let mut output = vec![0; number_of_bytes];
        self.cipher
            .try_write_keystream(&mut output)
            .map_err(stream_limit_error)?;
        Ok(output)
    }

    pub fn process_bytes(&mut self, input: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let mut output = input.to_vec();
        self.cipher
            .try_apply_keystream(&mut output)
            .map_err(stream_limit_error)?;
        Ok(output)
    }

    pub fn process_bytes_into(
        &mut self,
        input: &[u8],
        input_offset: usize,
        length: usize,
        output: &mut [u8],
        output_offset: usize,
    ) -> Result<usize, CryptoError> {
        process_bytes_into(
            &mut self.cipher,
            input,
            input_offset,
            length,
            output,
            output_offset,
        )
    }

    pub fn skip(&mut self, number_of_bytes: i64) -> Result<i64, CryptoError> {
        skip(&mut self.cipher, number_of_bytes)?;
        Ok(number_of_bytes)
    }

    pub fn seek_to(&mut self, position: u64) -> Result<u64, CryptoError> {
        self.cipher.try_seek(position).map_err(|_| {
            CryptoError::InvalidDataLength("Position exceeds stream length".to_owned())
        })?;
        Ok(position)
    }

    pub fn position(&self) -> u64 {
        self.cipher.current_pos()
    }
}

pub struct ChaChaEngine {
    cipher: ChaCha20Legacy,
}

impl ChaChaEngine {
    pub const ALGORITHM_NAME: &'static str = "ChaCha";

    pub fn new(key: &[u8], iv: &[u8]) -> Result<Self, CryptoError> {
        let key = chacha_key(key, Self::ALGORITHM_NAME)?;
        let iv = legacy_chacha_iv(iv)?;
        Ok(Self {
            cipher: ChaCha20Legacy::new(&key, &iv),
        })
    }

    pub fn get_bytes(&mut self, number_of_bytes: usize) -> Result<Vec<u8>, CryptoError> {
        let mut output = vec![0; number_of_bytes];
        self.cipher
            .try_write_keystream(&mut output)
            .map_err(stream_limit_error)?;
        Ok(output)
    }

    pub fn process_bytes(&mut self, input: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let mut output = input.to_vec();
        self.cipher
            .try_apply_keystream(&mut output)
            .map_err(stream_limit_error)?;
        Ok(output)
    }

    pub fn process_bytes_into(
        &mut self,
        input: &[u8],
        input_offset: usize,
        length: usize,
        output: &mut [u8],
        output_offset: usize,
    ) -> Result<usize, CryptoError> {
        process_bytes_into(
            &mut self.cipher,
            input,
            input_offset,
            length,
            output,
            output_offset,
        )
    }

    pub fn skip(&mut self, number_of_bytes: i64) -> Result<i64, CryptoError> {
        skip(&mut self.cipher, number_of_bytes)?;
        Ok(number_of_bytes)
    }

    pub fn seek_to(&mut self, position: u64) -> Result<u64, CryptoError> {
        self.cipher.try_seek(position).map_err(|_| {
            CryptoError::InvalidDataLength("Position exceeds stream length".to_owned())
        })?;
        Ok(position)
    }

    pub fn position(&self) -> u64 {
        self.cipher.current_pos()
    }
}

pub struct ChaCha7539Engine {
    cipher: ChaCha20,
}

impl ChaCha7539Engine {
    pub const ALGORITHM_NAME: &'static str = "ChaCha7539";

    pub fn new(key: &[u8], iv: &[u8]) -> Result<Self, CryptoError> {
        let key = chacha_key(key, Self::ALGORITHM_NAME)?;
        let iv = chacha7539_iv(iv)?;
        Ok(Self {
            cipher: ChaCha20::new(&key, &iv),
        })
    }

    pub fn get_bytes(&mut self, number_of_bytes: usize) -> Result<Vec<u8>, CryptoError> {
        let mut output = vec![0; number_of_bytes];
        self.cipher
            .try_write_keystream(&mut output)
            .map_err(stream_limit_error)?;
        Ok(output)
    }

    pub fn process_bytes(&mut self, input: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let mut output = input.to_vec();
        self.cipher
            .try_apply_keystream(&mut output)
            .map_err(stream_limit_error)?;
        Ok(output)
    }

    pub fn process_bytes_into(
        &mut self,
        input: &[u8],
        input_offset: usize,
        length: usize,
        output: &mut [u8],
        output_offset: usize,
    ) -> Result<usize, CryptoError> {
        process_bytes_into(
            &mut self.cipher,
            input,
            input_offset,
            length,
            output,
            output_offset,
        )
    }

    pub fn skip(&mut self, number_of_bytes: i64) -> Result<i64, CryptoError> {
        skip(&mut self.cipher, number_of_bytes)?;
        Ok(number_of_bytes)
    }

    pub fn seek_to(&mut self, position: u64) -> Result<u64, CryptoError> {
        self.cipher.try_seek(position).map_err(|_| {
            CryptoError::InvalidDataLength("Position exceeds stream length".to_owned())
        })?;
        Ok(position)
    }

    pub fn position(&self) -> u64 {
        self.cipher.current_pos()
    }
}

fn process_bytes_into<C>(
    cipher: &mut C,
    input: &[u8],
    input_offset: usize,
    length: usize,
    output: &mut [u8],
    output_offset: usize,
) -> Result<usize, CryptoError>
where
    C: StreamCipher,
{
    let input_end = input_offset
        .checked_add(length)
        .ok_or_else(|| CryptoError::InvalidDataLength("Input buffer too short".to_owned()))?;
    let output_end = output_offset
        .checked_add(length)
        .ok_or_else(|| CryptoError::InvalidDataLength("Output buffer too short".to_owned()))?;

    if input_end > input.len() {
        return Err(CryptoError::InvalidDataLength(
            "Input buffer too short".to_owned(),
        ));
    }
    if output_end > output.len() {
        return Err(CryptoError::InvalidDataLength(
            "Output buffer too short".to_owned(),
        ));
    }

    output[output_offset..output_end].copy_from_slice(&input[input_offset..input_end]);
    cipher
        .try_apply_keystream(&mut output[output_offset..output_end])
        .map_err(stream_limit_error)?;

    Ok(length)
}

fn skip<C>(cipher: &mut C, number_of_bytes: i64) -> Result<(), CryptoError>
where
    C: StreamCipherSeek,
{
    let position = cipher.current_pos::<u64>();
    let next_position = if number_of_bytes >= 0 {
        position.checked_add(number_of_bytes as u64)
    } else {
        position.checked_sub(number_of_bytes.unsigned_abs())
    }
    .ok_or_else(|| CryptoError::InvalidDataLength("Attempt to seek outside stream".to_owned()))?;

    cipher
        .try_seek(next_position)
        .map_err(|_| CryptoError::InvalidDataLength("Position exceeds stream length".to_owned()))
}

fn salsa_key(key: &[u8]) -> Result<SalsaKey, CryptoError> {
    if key.len() != 32 {
        return Err(CryptoError::InvalidKey(
            "Salsa20 requires 256 bit key".to_owned(),
        ));
    }
    SalsaKey::try_from(key)
        .map_err(|_| CryptoError::InvalidKey("Salsa20 requires 256 bit key".to_owned()))
}

fn salsa_iv(iv: &[u8]) -> Result<SalsaNonce, CryptoError> {
    if iv.len() != 8 {
        return Err(CryptoError::InvalidDataLength(
            "Salsa20 requires 64 bit IV".to_owned(),
        ));
    }
    SalsaNonce::try_from(iv)
        .map_err(|_| CryptoError::InvalidDataLength("Salsa20 requires 64 bit IV".to_owned()))
}

fn chacha_key(key: &[u8], algorithm_name: &str) -> Result<ChaChaKey, CryptoError> {
    if key.len() != 32 {
        return Err(CryptoError::InvalidKey(format!(
            "{algorithm_name} requires 256 bit key"
        )));
    }
    ChaChaKey::try_from(key)
        .map_err(|_| CryptoError::InvalidKey(format!("{algorithm_name} requires 256 bit key")))
}

fn legacy_chacha_iv(iv: &[u8]) -> Result<LegacyNonce, CryptoError> {
    if iv.len() != 8 {
        return Err(CryptoError::InvalidDataLength(
            "ChaCha requires 64 bit IV".to_owned(),
        ));
    }
    LegacyNonce::try_from(iv)
        .map_err(|_| CryptoError::InvalidDataLength("ChaCha requires 64 bit IV".to_owned()))
}

fn chacha7539_iv(iv: &[u8]) -> Result<ChaChaNonce, CryptoError> {
    if iv.len() != 12 {
        return Err(CryptoError::InvalidDataLength(
            "ChaCha7539 requires 96 bit IV".to_owned(),
        ));
    }
    ChaChaNonce::try_from(iv)
        .map_err(|_| CryptoError::InvalidDataLength("ChaCha7539 requires 96 bit IV".to_owned()))
}

fn stream_limit_error(_: cipher::StreamCipherError) -> CryptoError {
    CryptoError::MaxBytesExceeded("2^70 byte limit per IV would be exceeded; Change IV".to_owned())
}

#[cfg(test)]
mod tests {
    use crate::crypto::{
        byte_array::constant_time_equals,
        stream::{ChaCha7539Engine, ChaChaEngine, Salsa20Engine, chacha_core},
    };

    #[test]
    fn chacha_core_matches_rfc8439_block_vector() {
        let input = [
            0x61707865, 0x3320646e, 0x79622d32, 0x6b206574, 0x03020100, 0x07060504, 0x0b0a0908,
            0x0f0e0d0c, 0x13121110, 0x17161514, 0x1b1a1918, 0x1f1e1d1c, 0x00000001, 0x09000000,
            0x4a000000, 0x00000000,
        ];

        assert_eq!(
            chacha_core(20, &input),
            [
                0xe4e7f110, 0x15593bd1, 0x1fdd0f50, 0xc47120a3, 0xc7f4d1c7, 0x0368c033, 0x9aaa2204,
                0x4e6cd4c3, 0x466482d2, 0x09aa9f07, 0x05d7c214, 0xa2028bd9, 0xd19c12b5, 0xb94e16de,
                0xe883d0cb, 0x4e3c50a2,
            ]
        );
    }

    #[test]
    fn salsa20_round_trips_and_seeks() {
        let key = [7; 32];
        let iv = [3; 8];
        let plain = b"stream cipher plaintext";

        let mut enc = Salsa20Engine::new(&key, &iv).unwrap();
        let encrypted = enc.process_bytes(plain).unwrap();

        let mut dec = Salsa20Engine::new(&key, &iv).unwrap();
        assert_eq!(dec.process_bytes(&encrypted).unwrap(), plain);

        let mut all = Salsa20Engine::new(&key, &iv).unwrap();
        let first = all.get_bytes(9).unwrap();
        let mut seeked = Salsa20Engine::new(&key, &iv).unwrap();
        seeked.seek_to(4).unwrap();
        assert_eq!(seeked.get_bytes(5).unwrap(), first[4..]);
    }

    #[test]
    fn chacha_variants_round_trip() {
        let key = [11; 32];
        let legacy_iv = [5; 8];
        let ietf_iv = [9; 12];
        let plain = b"chacha plaintext";

        let mut legacy_enc = ChaChaEngine::new(&key, &legacy_iv).unwrap();
        let encrypted = legacy_enc.process_bytes(plain).unwrap();
        let mut legacy_dec = ChaChaEngine::new(&key, &legacy_iv).unwrap();
        assert!(constant_time_equals(
            &legacy_dec.process_bytes(&encrypted).unwrap(),
            plain
        ));

        let mut ietf_enc = ChaCha7539Engine::new(&key, &ietf_iv).unwrap();
        let encrypted = ietf_enc.process_bytes(plain).unwrap();
        let mut ietf_dec = ChaCha7539Engine::new(&key, &ietf_iv).unwrap();
        assert_eq!(ietf_dec.process_bytes(&encrypted).unwrap(), plain);
    }
}
