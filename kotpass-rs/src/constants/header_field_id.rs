#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum HeaderFieldId {
    /// Marks the end of the header fields section.
    EndOfHeader = 0,

    /// Legacy comment field, deprecated and no longer used.
    Comment = 1,

    /// Identifier for the encryption cipher used.
    CipherId = 2,

    /// Compression algorithm identifier.
    Compression = 3,

    /// Random seed used for master key derivation.
    MasterSeed = 4,

    /// Random seed used for legacy AES-KDF key transformation.
    TransformSeed = 5,

    /// Number of legacy AES-KDF transformation rounds.
    TransformRounds = 6,

    /// Initialization vector for the encryption cipher.
    EncryptionIv = 7,

    /// Key used for inner random stream encryption.
    InnerRandomStreamKey = 8,

    /// First bytes of decrypted data used to verify correct decryption.
    StreamStartBytes = 9,

    /// Identifier for inner random stream algorithm.
    InnerRandomStreamId = 10,

    /// Parameters for modern key derivation functions stored as a variant dictionary.
    KdfParameters = 11,

    /// Custom data that can be read by third-party applications.
    PublicCustomData = 12,
}

impl HeaderFieldId {
    pub const ALL: [Self; 13] = [
        Self::EndOfHeader,
        Self::Comment,
        Self::CipherId,
        Self::Compression,
        Self::MasterSeed,
        Self::TransformSeed,
        Self::TransformRounds,
        Self::EncryptionIv,
        Self::InnerRandomStreamKey,
        Self::StreamStartBytes,
        Self::InnerRandomStreamId,
        Self::KdfParameters,
        Self::PublicCustomData,
    ];

    pub const fn id(self) -> u8 {
        self as u8
    }

    pub fn from_id(id: u8) -> Option<Self> {
        Self::ALL.get(id as usize).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::HeaderFieldId;

    #[test]
    fn ids_match_kotlin_enum_ordinals() {
        assert_eq!(HeaderFieldId::EndOfHeader.id(), 0);
        assert_eq!(HeaderFieldId::CipherId.id(), 2);
        assert_eq!(HeaderFieldId::EncryptionIv.id(), 7);
        assert_eq!(HeaderFieldId::PublicCustomData.id(), 12);
    }

    #[test]
    fn parses_known_ids() {
        assert_eq!(HeaderFieldId::from_id(0), Some(HeaderFieldId::EndOfHeader));
        assert_eq!(
            HeaderFieldId::from_id(11),
            Some(HeaderFieldId::KdfParameters)
        );
        assert_eq!(HeaderFieldId::from_id(13), None);
    }
}
