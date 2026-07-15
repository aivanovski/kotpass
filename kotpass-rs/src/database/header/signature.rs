use std::io::{self, Write};

use crate::io::buffered_stream::BufferedStream;

/// Identifies a KeePass database file type before the format version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Signature {
    pub base: [u8; 4],
    pub secondary: [u8; 4],
}

impl Signature {
    pub const BASE: [u8; 4] = [0x03, 0xD9, 0xA2, 0x9A];
    pub const SECONDARY: [u8; 4] = [0x67, 0xFB, 0x4B, 0xB5];
    pub const DEFAULT: Self = Self::new(Self::BASE, Self::SECONDARY);

    pub const fn new(base: [u8; 4], secondary: [u8; 4]) -> Self {
        Self { base, secondary }
    }

    pub fn write_to<W: Write>(&self, sink: &mut W) -> io::Result<()> {
        sink.write_all(&self.base)?;
        sink.write_all(&self.secondary)?;
        Ok(())
    }

    pub fn read_from<S: BufferedStream>(source: &mut S) -> io::Result<Self> {
        Ok(Self {
            base: source.read_array()?,
            secondary: source.read_array()?,
        })
    }
}

impl Default for Signature {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
mod tests {
    use super::Signature;
    use crate::io::real_buffered_stream::RealBufferedStream;

    #[test]
    fn default_matches_kotlin_constants() {
        assert_eq!(Signature::BASE, [0x03, 0xD9, 0xA2, 0x9A]);
        assert_eq!(Signature::SECONDARY, [0x67, 0xFB, 0x4B, 0xB5]);
        assert_eq!(
            Signature::DEFAULT,
            Signature {
                base: Signature::BASE,
                secondary: Signature::SECONDARY,
            }
        );
    }

    #[test]
    fn reads_and_writes_base_then_secondary() {
        let signature = Signature::new([1, 2, 3, 4], [5, 6, 7, 8]);
        let mut bytes = Vec::new();

        signature.write_to(&mut bytes).unwrap();

        assert_eq!(bytes, vec![1, 2, 3, 4, 5, 6, 7, 8]);
        let mut source = RealBufferedStream::from_bytes(bytes);
        assert_eq!(Signature::read_from(&mut source).unwrap(), signature);
    }

    #[test]
    fn reports_short_sources_as_unexpected_eof() {
        let mut source = RealBufferedStream::from_bytes(vec![1, 2, 3]);

        let error = Signature::read_from(&mut source).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof);
    }
}
