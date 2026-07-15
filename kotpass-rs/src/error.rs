use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CryptoError {
    #[error("{0}")]
    InvalidDataLength(String),

    #[error("{0}")]
    MaxBytesExceeded(String),

    #[error("{0}")]
    AlgorithmUnavailable(String),

    #[error("{0}")]
    InvalidKey(String),

    #[error("{0}")]
    InvalidCipherText(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FormatError {
    #[error("{0}")]
    UnknownFormat(String),

    #[error("{0}")]
    UnsupportedVersion(String),

    #[error("{0}")]
    InvalidHeader(String),

    #[error("{0}")]
    InvalidContent(String),

    #[error("{0}")]
    InvalidXml(String),

    #[error("{0}")]
    FailedCompression(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum KeyfileError {
    #[error("invalid keyfile version")]
    InvalidVersion,

    #[error("invalid keyfile hash")]
    InvalidHash,

    #[error("keyfile contains no key data")]
    NoKeyData,
}

#[cfg(test)]
mod tests {
    use super::{CryptoError, FormatError, KeyfileError};

    #[test]
    fn crypto_error_display_preserves_message() {
        let error = CryptoError::InvalidKey("Wrong KDF seed used for decryption.".to_owned());

        assert_eq!(error.to_string(), "Wrong KDF seed used for decryption.");
    }

    #[test]
    fn format_error_display_preserves_message() {
        let error = FormatError::InvalidHeader("No KDF UUID found.".to_owned());

        assert_eq!(error.to_string(), "No KDF UUID found.");
    }

    #[test]
    fn keyfile_error_variants_are_distinct() {
        assert_ne!(KeyfileError::InvalidVersion, KeyfileError::InvalidHash);
        assert_ne!(KeyfileError::InvalidHash, KeyfileError::NoKeyData);
    }
}
