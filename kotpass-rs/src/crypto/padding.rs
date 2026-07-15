use crate::error::CryptoError;

pub trait BlockCipherPadding {
    fn add_padding(&self, input: &mut [u8], offset: usize) -> usize;
    fn pad_count(&self, input: &[u8]) -> Result<usize, CryptoError>;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Pkcs7Padding;

impl BlockCipherPadding for Pkcs7Padding {
    fn add_padding(&self, input: &mut [u8], offset: usize) -> usize {
        assert!(offset <= input.len(), "padding offset exceeds input length");

        let code = input.len() - offset;
        input[offset..].fill(code as u8);
        code
    }

    fn pad_count(&self, input: &[u8]) -> Result<usize, CryptoError> {
        if input.is_empty() {
            return Err(CryptoError::InvalidCipherText(
                "Pad block is corrupted".to_owned(),
            ));
        }

        let count = input[input.len() - 1] as usize;
        let position = input.len().wrapping_sub(count);
        let mut failed = ((position | count.wrapping_sub(1)) >> (usize::BITS - 1)) as u8;

        for (i, byte) in input.iter().enumerate() {
            let mask = if i >= position { 0xff } else { 0x00 };
            failed |= (byte ^ count as u8) & mask;
        }

        if failed != 0 {
            return Err(CryptoError::InvalidCipherText(
                "Pad block is corrupted".to_owned(),
            ));
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::padding::{BlockCipherPadding, Pkcs7Padding};

    #[test]
    fn adds_pkcs7_padding() {
        let mut block = *b"YELLOW SUBMARINE";

        assert_eq!(Pkcs7Padding.add_padding(&mut block, 11), 5);
        assert_eq!(&block, b"YELLOW SUBM\x05\x05\x05\x05\x05");
    }

    #[test]
    fn counts_valid_pkcs7_padding() {
        assert_eq!(
            Pkcs7Padding.pad_count(b"ICE ICE BABY\x04\x04\x04\x04"),
            Ok(4)
        );
    }

    #[test]
    fn rejects_corrupt_pkcs7_padding() {
        assert!(
            Pkcs7Padding
                .pad_count(b"ICE ICE BABY\x05\x05\x05\x05")
                .is_err()
        );
        assert!(
            Pkcs7Padding
                .pad_count(b"ICE ICE BABY\x01\x02\x03\x04")
                .is_err()
        );
    }
}
