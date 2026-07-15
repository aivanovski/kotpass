/// Specifies the stream cipher for in-memory protection of sensitive data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(usize)]
pub enum CrsAlgorithm {
    /// Unsupported.
    #[default]
    None = 0,

    /// Legacy ArcFour (RC4) variant. Unsupported.
    ArcFourVariant = 1,

    /// Salsa20 stream cipher.
    Salsa20 = 2,

    /// ChaCha20 stream cipher.
    ChaCha20 = 3,
}

impl CrsAlgorithm {
    pub const ALL: [Self; 4] = [
        Self::None,
        Self::ArcFourVariant,
        Self::Salsa20,
        Self::ChaCha20,
    ];

    /// Returns the Kotlin enum ordinal stored in KDBX headers.
    pub const fn ordinal(self) -> usize {
        self as usize
    }

    /// Converts a KDBX header ordinal into a stream cipher identifier.
    pub fn from_ordinal(ordinal: usize) -> Option<Self> {
        Self::ALL.get(ordinal).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::CrsAlgorithm;

    #[test]
    fn ordinals_match_kotlin_enum_order() {
        assert_eq!(CrsAlgorithm::None.ordinal(), 0);
        assert_eq!(CrsAlgorithm::ArcFourVariant.ordinal(), 1);
        assert_eq!(CrsAlgorithm::Salsa20.ordinal(), 2);
        assert_eq!(CrsAlgorithm::ChaCha20.ordinal(), 3);
    }

    #[test]
    fn parses_known_ordinals() {
        assert_eq!(CrsAlgorithm::from_ordinal(0), Some(CrsAlgorithm::None));
        assert_eq!(CrsAlgorithm::from_ordinal(2), Some(CrsAlgorithm::Salsa20));
        assert_eq!(CrsAlgorithm::from_ordinal(3), Some(CrsAlgorithm::ChaCha20));
        assert_eq!(CrsAlgorithm::from_ordinal(4), None);
    }
}
