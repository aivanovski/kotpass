use ::cipher::KeyInit;

use crate::{crypto::padding::BlockCipherPadding, error::CryptoError};

#[derive(Clone)]
pub struct TwofishEngine {
    cipher: Option<twofish::Twofish>,
    encrypting: bool,
    working_key: Option<Vec<u8>>,
}

impl TwofishEngine {
    pub const BLOCK_SIZE: usize = 16;

    pub fn new() -> Self {
        Self {
            cipher: None,
            encrypting: false,
            working_key: None,
        }
    }
}

impl Default for TwofishEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockCipher for TwofishEngine {
    fn block_size(&self) -> usize {
        Self::BLOCK_SIZE
    }

    fn is_encrypting(&self) -> bool {
        self.encrypting
    }

    fn init(&mut self, encrypting: bool, key: &[u8]) -> Result<(), CryptoError> {
        if !matches!(key.len(), 16 | 24 | 32) {
            return Err(CryptoError::InvalidDataLength(
                "Twofish key length must be 128/192/256 bits".to_owned(),
            ));
        }

        self.encrypting = encrypting;
        self.working_key = Some(key.to_vec());
        self.cipher = Some(twofish::Twofish::new_from_slice(key).map_err(|_| {
            CryptoError::InvalidDataLength("Twofish key length must be 128/192/256 bits".to_owned())
        })?);
        Ok(())
    }

    fn process_block(
        &mut self,
        src: &[u8],
        src_offset: usize,
        dst: &mut [u8],
        dst_offset: usize,
    ) -> Result<usize, CryptoError> {
        let cipher = self
            .cipher
            .as_ref()
            .ok_or_else(|| CryptoError::InvalidKey("Twofish is not initialised".to_owned()))?;

        ensure_input(src, src_offset, Self::BLOCK_SIZE)?;
        if dst_offset
            .checked_add(Self::BLOCK_SIZE)
            .is_none_or(|end| end > dst.len())
        {
            return Err(CryptoError::InvalidDataLength(
                "Output buffer is too short".to_owned(),
            ));
        }

        let mut block = ::cipher::Block::<twofish::Twofish>::default();
        block.copy_from_slice(&src[src_offset..src_offset + Self::BLOCK_SIZE]);

        if self.encrypting {
            ::cipher::BlockCipherEncrypt::encrypt_block(cipher, &mut block);
        } else {
            ::cipher::BlockCipherDecrypt::decrypt_block(cipher, &mut block);
        }

        dst[dst_offset..dst_offset + Self::BLOCK_SIZE].copy_from_slice(&block);
        Ok(Self::BLOCK_SIZE)
    }

    fn reset(&mut self) {
        if let Some(key) = &self.working_key {
            self.cipher = twofish::Twofish::new_from_slice(key).ok();
        }
    }
}

pub trait BlockCipher {
    fn block_size(&self) -> usize;
    fn is_encrypting(&self) -> bool;
    fn init(&mut self, encrypting: bool, key: &[u8]) -> Result<(), CryptoError>;
    fn process_block(
        &mut self,
        src: &[u8],
        src_offset: usize,
        dst: &mut [u8],
        dst_offset: usize,
    ) -> Result<usize, CryptoError>;
    fn reset(&mut self);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BlockCipherMode {
    Cbc(CbcBlockCipherMode),
}

impl BlockCipherMode {
    pub fn cbc(iv: impl Into<Vec<u8>>) -> Self {
        Self::Cbc(CbcBlockCipherMode::new(iv))
    }

    pub fn process_block<C: BlockCipher>(
        &mut self,
        cipher: &mut C,
        src: &[u8],
        src_offset: usize,
        dst: &mut [u8],
        dst_offset: usize,
    ) -> Result<usize, CryptoError> {
        match self {
            Self::Cbc(cbc) => cbc.process_block(cipher, src, src_offset, dst, dst_offset),
        }
    }

    pub fn reset(&mut self) {
        match self {
            Self::Cbc(cbc) => cbc.reset(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CbcBlockCipherMode {
    iv: Vec<u8>,
    cbc_v: Vec<u8>,
    cbc_next_v: Vec<u8>,
}

impl CbcBlockCipherMode {
    pub fn new(iv: impl Into<Vec<u8>>) -> Self {
        let iv = iv.into();
        Self {
            cbc_v: iv.clone(),
            cbc_next_v: vec![0; iv.len()],
            iv,
        }
    }

    pub fn process_block<C: BlockCipher>(
        &mut self,
        cipher: &mut C,
        src: &[u8],
        src_offset: usize,
        dst: &mut [u8],
        dst_offset: usize,
    ) -> Result<usize, CryptoError> {
        if cipher.is_encrypting() {
            self.encrypt_block(cipher, src, src_offset, dst, dst_offset)
        } else {
            self.decrypt_block(cipher, src, src_offset, dst, dst_offset)
        }
    }

    pub fn reset(&mut self) {
        self.cbc_v.copy_from_slice(&self.iv);
        self.cbc_next_v.fill(0);
    }

    fn encrypt_block<C: BlockCipher>(
        &mut self,
        cipher: &mut C,
        src: &[u8],
        src_offset: usize,
        dst: &mut [u8],
        dst_offset: usize,
    ) -> Result<usize, CryptoError> {
        let block_size = cipher.block_size();
        ensure_input(src, src_offset, block_size)?;

        for i in 0..block_size {
            self.cbc_v[i] ^= src[src_offset + i];
        }

        let length = cipher.process_block(&self.cbc_v, 0, dst, dst_offset)?;
        let cbc_len = self.cbc_v.len();
        self.cbc_v
            .copy_from_slice(&dst[dst_offset..dst_offset + cbc_len]);

        Ok(length)
    }

    fn decrypt_block<C: BlockCipher>(
        &mut self,
        cipher: &mut C,
        src: &[u8],
        src_offset: usize,
        dst: &mut [u8],
        dst_offset: usize,
    ) -> Result<usize, CryptoError> {
        let block_size = cipher.block_size();
        ensure_input(src, src_offset, block_size)?;

        self.cbc_next_v
            .copy_from_slice(&src[src_offset..src_offset + block_size]);

        let length = cipher.process_block(src, src_offset, dst, dst_offset)?;

        for i in 0..block_size {
            dst[dst_offset + i] ^= self.cbc_v[i];
        }

        std::mem::swap(&mut self.cbc_v, &mut self.cbc_next_v);

        Ok(length)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PaddedBufferedBlockCipher<C, P> {
    cipher: C,
    mode: BlockCipherMode,
    padding: P,
    buf: Vec<u8>,
    buf_off: usize,
    for_encryption: bool,
}

impl<C, P> PaddedBufferedBlockCipher<C, P>
where
    C: BlockCipher,
    P: BlockCipherPadding,
{
    pub fn new(cipher: C, mode: BlockCipherMode, padding: P) -> Self {
        let block_size = cipher.block_size();
        Self {
            cipher,
            mode,
            padding,
            buf: vec![0; block_size],
            buf_off: 0,
            for_encryption: false,
        }
    }

    pub fn init(&mut self, encryption: bool, key: &[u8]) -> Result<(), CryptoError> {
        self.for_encryption = encryption;
        self.reset();
        self.cipher.init(encryption, key)
    }

    pub fn output_size(&self, len: usize) -> usize {
        let total = len + self.buf_off;
        let left_over = total % self.buf.len();

        if left_over == 0 && self.for_encryption {
            total + self.buf.len()
        } else if left_over == 0 {
            total
        } else {
            total - left_over + self.buf.len()
        }
    }

    pub fn update_output_size(&self, len: usize) -> usize {
        let total = len + self.buf_off;
        let left_over = total % self.buf.len();

        if left_over == 0 {
            total.saturating_sub(self.buf.len())
        } else {
            total - left_over
        }
    }

    pub fn process_bytes_to_vec(&mut self, src: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let mut buf = vec![0; self.output_size(src.len())];
        let mut len = self.process_bytes(src, 0, src.len(), &mut buf, 0)?;
        len += self.do_final(&mut buf, len)?;
        buf.truncate(len);
        Ok(buf)
    }

    pub fn process_bytes(
        &mut self,
        src: &[u8],
        src_offset: usize,
        len: usize,
        dst: &mut [u8],
        dst_offset: usize,
    ) -> Result<usize, CryptoError> {
        ensure_input(src, src_offset, len)?;

        let block_size = self.cipher.block_size();
        let length = self.update_output_size(len);

        if length > 0 && dst_offset + length > dst.len() {
            return Err(CryptoError::InvalidDataLength(
                "Output buffer is too short".to_owned(),
            ));
        }

        let mut src_off = src_offset;
        let mut len = len;
        let mut result_len = 0;
        let gap_len = self.buf.len() - self.buf_off;

        if len > gap_len {
            self.buf[self.buf_off..self.buf_off + gap_len]
                .copy_from_slice(&src[src_off..src_off + gap_len]);

            result_len +=
                self.mode
                    .process_block(&mut self.cipher, &self.buf, 0, dst, dst_offset)?;

            self.buf_off = 0;
            len -= gap_len;
            src_off += gap_len;

            while len > self.buf.len() {
                result_len += self.mode.process_block(
                    &mut self.cipher,
                    src,
                    src_off,
                    dst,
                    dst_offset + result_len,
                )?;

                len -= block_size;
                src_off += block_size;
            }
        }

        self.buf[self.buf_off..self.buf_off + len].copy_from_slice(&src[src_off..src_off + len]);
        self.buf_off += len;

        Ok(result_len)
    }

    pub fn do_final(&mut self, dst: &mut [u8], dst_offset: usize) -> Result<usize, CryptoError> {
        let mut result_len = 0;

        if self.for_encryption {
            if self.buf_off == self.cipher.block_size() {
                if dst_offset + 2 * self.cipher.block_size() > dst.len() {
                    self.reset();
                    return Err(CryptoError::InvalidDataLength(
                        "Output buffer is too short".to_owned(),
                    ));
                }

                result_len =
                    self.mode
                        .process_block(&mut self.cipher, &self.buf, 0, dst, dst_offset)?;
                self.buf_off = 0;
            }

            self.padding.add_padding(&mut self.buf, self.buf_off);

            result_len += self.mode.process_block(
                &mut self.cipher,
                &self.buf,
                0,
                dst,
                dst_offset + result_len,
            )?;

            self.reset();
            Ok(result_len)
        } else if self.buf_off == self.cipher.block_size() {
            let mut final_block = vec![0; self.buf.len()];
            result_len =
                self.mode
                    .process_block(&mut self.cipher, &self.buf, 0, &mut final_block, 0)?;
            self.buf_off = 0;

            let pad_count = self.padding.pad_count(&final_block)?;
            result_len -= pad_count;
            dst[dst_offset..dst_offset + result_len].copy_from_slice(&final_block[..result_len]);
            self.reset();
            Ok(result_len)
        } else {
            self.reset();
            Err(CryptoError::InvalidDataLength(
                "Last block incomplete in decryption".to_owned(),
            ))
        }
    }

    pub fn reset(&mut self) {
        self.buf.fill(0);
        self.buf_off = 0;
        self.cipher.reset();
        self.mode.reset();
    }

    pub fn into_inner(self) -> C {
        self.cipher
    }
}

fn ensure_input(src: &[u8], offset: usize, len: usize) -> Result<(), CryptoError> {
    if offset.checked_add(len).is_none_or(|end| end > src.len()) {
        Err(CryptoError::InvalidDataLength(
            "Input buffer is too short".to_owned(),
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        crypto::{
            cipher::{BlockCipher, BlockCipherMode, PaddedBufferedBlockCipher, TwofishEngine},
            padding::Pkcs7Padding,
        },
        error::CryptoError,
    };

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct XorBlockCipher {
        encrypting: bool,
        key: Vec<u8>,
    }

    impl XorBlockCipher {
        fn new() -> Self {
            Self {
                encrypting: false,
                key: vec![0; 4],
            }
        }
    }

    impl BlockCipher for XorBlockCipher {
        fn block_size(&self) -> usize {
            4
        }

        fn is_encrypting(&self) -> bool {
            self.encrypting
        }

        fn init(&mut self, encrypting: bool, key: &[u8]) -> Result<(), CryptoError> {
            self.encrypting = encrypting;
            self.key = key.to_vec();
            Ok(())
        }

        fn process_block(
            &mut self,
            src: &[u8],
            src_offset: usize,
            dst: &mut [u8],
            dst_offset: usize,
        ) -> Result<usize, CryptoError> {
            for i in 0..4 {
                dst[dst_offset + i] = src[src_offset + i] ^ self.key[i];
            }
            Ok(4)
        }

        fn reset(&mut self) {}
    }

    #[test]
    fn cbc_encrypts_and_decrypts_blocks() {
        let key = [0x10, 0x20, 0x30, 0x40];
        let iv = [1, 2, 3, 4];
        let plain = *b"abcdefgh";
        let mut encrypted = [0; 8];

        let mut enc = XorBlockCipher::new();
        enc.init(true, &key).unwrap();
        let mut enc_mode = BlockCipherMode::cbc(iv);
        enc_mode
            .process_block(&mut enc, &plain, 0, &mut encrypted, 0)
            .unwrap();
        enc_mode
            .process_block(&mut enc, &plain, 4, &mut encrypted, 4)
            .unwrap();

        let mut decrypted = [0; 8];
        let mut dec = XorBlockCipher::new();
        dec.init(false, &key).unwrap();
        let mut dec_mode = BlockCipherMode::cbc(iv);
        dec_mode
            .process_block(&mut dec, &encrypted, 0, &mut decrypted, 0)
            .unwrap();
        dec_mode
            .process_block(&mut dec, &encrypted, 4, &mut decrypted, 4)
            .unwrap();

        assert_eq!(decrypted, plain);
    }

    #[test]
    fn buffered_cipher_round_trips_with_pkcs7_padding() {
        let key = [0x10, 0x20, 0x30, 0x40];
        let iv = [1, 2, 3, 4];
        let plain = b"hello world";

        let mut enc = PaddedBufferedBlockCipher::new(
            XorBlockCipher::new(),
            BlockCipherMode::cbc(iv),
            Pkcs7Padding,
        );
        enc.init(true, &key).unwrap();
        let encrypted = enc.process_bytes_to_vec(plain).unwrap();

        let mut dec = PaddedBufferedBlockCipher::new(
            XorBlockCipher::new(),
            BlockCipherMode::cbc(iv),
            Pkcs7Padding,
        );
        dec.init(false, &key).unwrap();
        let decrypted = dec.process_bytes_to_vec(&encrypted).unwrap();

        assert_eq!(decrypted, plain);
    }

    #[test]
    fn twofish_matches_standard_zero_key_vector() {
        let mut cipher = TwofishEngine::new();
        let mut encrypted = [0; 16];
        cipher.init(true, &[0; 16]).unwrap();
        cipher
            .process_block(&[0; 16], 0, &mut encrypted, 0)
            .unwrap();

        assert_eq!(hex::encode(encrypted), "9f589f5cf6122c32b6bfec2f2ae8c35a");

        let mut decrypted = [0xff; 16];
        cipher.init(false, &[0; 16]).unwrap();
        cipher
            .process_block(&encrypted, 0, &mut decrypted, 0)
            .unwrap();

        assert_eq!(decrypted, [0; 16]);
    }
}
