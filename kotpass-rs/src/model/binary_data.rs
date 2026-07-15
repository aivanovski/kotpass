use std::io::{Read, Write};

use flate2::{Compression, read::GzDecoder, write::GzEncoder};

use crate::{crypto::byte_array::sha256, error::FormatError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BinaryData {
    Uncompressed {
        memory_protection: bool,
        raw_content: Vec<u8>,
        hash: Vec<u8>,
    },
    Compressed {
        memory_protection: bool,
        raw_content: Vec<u8>,
        hash: Vec<u8>,
    },
}

impl BinaryData {
    pub fn uncompressed(memory_protection: bool, raw_content: impl Into<Vec<u8>>) -> Self {
        let raw_content = raw_content.into();
        let hash = sha256(&raw_content);
        Self::Uncompressed {
            memory_protection,
            raw_content,
            hash,
        }
    }

    pub fn compressed(memory_protection: bool, raw_content: impl Into<Vec<u8>>) -> Self {
        let raw_content = raw_content.into();
        let hash = sha256(&raw_content);
        Self::Compressed {
            memory_protection,
            raw_content,
            hash,
        }
    }

    pub fn hash(&self) -> &[u8] {
        match self {
            Self::Uncompressed { hash, .. } | Self::Compressed { hash, .. } => hash,
        }
    }

    pub fn memory_protection(&self) -> bool {
        match self {
            Self::Uncompressed {
                memory_protection, ..
            }
            | Self::Compressed {
                memory_protection, ..
            } => *memory_protection,
        }
    }

    pub fn raw_content(&self) -> &[u8] {
        match self {
            Self::Uncompressed { raw_content, .. } | Self::Compressed { raw_content, .. } => {
                raw_content
            }
        }
    }

    pub fn get_content(&self) -> Result<Vec<u8>, FormatError> {
        match self {
            Self::Uncompressed { raw_content, .. } => Ok(raw_content.clone()),
            Self::Compressed { raw_content, .. } => {
                let mut decoder = GzDecoder::new(raw_content.as_slice());
                let mut content = Vec::new();
                decoder.read_to_end(&mut content).map_err(|_| {
                    FormatError::FailedCompression(
                        "Failed to read from compressed binary data stream.".to_owned(),
                    )
                })?;
                Ok(content)
            }
        }
    }

    pub fn to_compressed(&self) -> Result<Self, FormatError> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&self.get_content()?).map_err(|_| {
            FormatError::FailedCompression("Failed to gzip binary data.".to_owned())
        })?;
        let compressed = encoder.finish().map_err(|_| {
            FormatError::FailedCompression("Failed to gzip binary data.".to_owned())
        })?;

        Ok(Self::compressed(self.memory_protection(), compressed))
    }
}

#[cfg(test)]
mod tests {
    use super::BinaryData;

    #[test]
    fn uncompressed_content_is_raw_content() {
        let data = BinaryData::uncompressed(true, b"content".to_vec());

        assert_eq!(data.get_content().unwrap(), b"content");
        assert!(data.memory_protection());
    }

    #[test]
    fn compresses_and_decompresses_content() {
        let data = BinaryData::uncompressed(false, b"content".to_vec())
            .to_compressed()
            .unwrap();

        assert_eq!(data.get_content().unwrap(), b"content");
        assert!(!data.memory_protection());
    }
}
