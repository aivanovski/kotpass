/// Specifies the obfuscation method for Auto-Type to protect against keyloggers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AutoTypeObfuscation {
    /// Sends characters as individual keystrokes.
    ///
    /// This is less secure against keyloggers.
    #[default]
    None,

    /// Pastes the password via the system clipboard to bypass keyloggers.
    ///
    /// Also known as Two-Channel Auto-Type Obfuscation (TCATO).
    UseClipboard,
}

impl AutoTypeObfuscation {
    /// Returns the Kotlin enum ordinal used by KeePass XML.
    pub const fn ordinal(self) -> usize {
        match self {
            Self::None => 0,
            Self::UseClipboard => 1,
        }
    }

    /// Converts a KeePass XML ordinal into an obfuscation mode.
    pub const fn from_ordinal(ordinal: usize) -> Option<Self> {
        match ordinal {
            0 => Some(Self::None),
            1 => Some(Self::UseClipboard),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AutoTypeObfuscation;

    #[test]
    fn default_matches_kotlin_model_default() {
        assert_eq!(AutoTypeObfuscation::default(), AutoTypeObfuscation::None);
    }

    #[test]
    fn ordinals_match_kotlin_enum_order() {
        assert_eq!(AutoTypeObfuscation::None.ordinal(), 0);
        assert_eq!(AutoTypeObfuscation::UseClipboard.ordinal(), 1);
    }

    #[test]
    fn parses_known_ordinals() {
        assert_eq!(
            AutoTypeObfuscation::from_ordinal(0),
            Some(AutoTypeObfuscation::None)
        );
        assert_eq!(
            AutoTypeObfuscation::from_ordinal(1),
            Some(AutoTypeObfuscation::UseClipboard)
        );
        assert_eq!(AutoTypeObfuscation::from_ordinal(2), None);
    }
}
