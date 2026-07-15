use std::io::{self, Write};

use crate::io::buffered_stream::BufferedStream;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FormatVersion {
    pub major: i16,
    pub minor: i16,
}

impl FormatVersion {
    pub const fn new(major: i16, minor: i16) -> Self {
        Self { major, minor }
    }

    pub const fn is_at_least(self, major: i16, minor: i16) -> bool {
        self.major > major || (self.major == major && self.minor >= minor)
    }

    pub fn write_to<W: Write>(&self, sink: &mut W) -> io::Result<()> {
        sink.write_all(&self.minor.to_le_bytes())?;
        sink.write_all(&self.major.to_le_bytes())?;
        Ok(())
    }

    pub fn read_from<S: BufferedStream>(source: &mut S) -> io::Result<Self> {
        Ok(Self {
            minor: source.read_short_le()?,
            major: source.read_short_le()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{io::real_buffered_stream::RealBufferedStream, model::FormatVersion};

    #[test]
    fn compares_versions_like_kotlin() {
        let version = FormatVersion::new(4, 1);

        assert!(version.is_at_least(4, 0));
        assert!(version.is_at_least(3, 9));
        assert!(!version.is_at_least(4, 2));
    }

    #[test]
    fn reads_and_writes_minor_then_major_little_endian() {
        let version = FormatVersion::new(4, 1);
        let mut bytes = Vec::new();

        version.write_to(&mut bytes).unwrap();

        assert_eq!(bytes, vec![1, 0, 4, 0]);
        let mut source = RealBufferedStream::from_bytes(bytes);
        assert_eq!(FormatVersion::read_from(&mut source).unwrap(), version);
    }
}
