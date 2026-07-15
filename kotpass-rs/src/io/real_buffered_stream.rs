use std::io::{self, Read};

use super::buffered_stream::BufferedStream;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealBufferedStream {
    bytes: Vec<u8>,
    position: usize,
}

impl RealBufferedStream {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { bytes, position: 0 }
    }

    pub fn from_reader<R: Read>(mut reader: R) -> io::Result<Self> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        Ok(Self::from_bytes(bytes))
    }

    pub fn buffer(&self) -> &[u8] {
        self.remaining()
    }
}

impl BufferedStream for RealBufferedStream {
    fn remaining(&self) -> &[u8] {
        &self.bytes[self.position..]
    }

    fn advance(&mut self, byte_count: usize) {
        self.position += byte_count;
    }
}

impl Read for RealBufferedStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        BufferedStream::read_into(self, buf)
    }
}

#[cfg(test)]
mod tests {
    use super::RealBufferedStream;
    use crate::io::buffered_stream::BufferedStream;

    #[test]
    fn reads_primitive_values() {
        let mut stream = RealBufferedStream::from_bytes(vec![
            0x01, 0x02, 0x04, 0x03, 0x00, 0x00, 0x00, 0x05, 0x08, 0x07, 0x06, 0x05, 0x04, 0x03,
            0x02, 0x01,
        ]);

        assert_eq!(stream.read_short().unwrap(), 0x0102);
        assert_eq!(stream.read_short_le().unwrap(), 0x0304);
        assert_eq!(stream.read_int().unwrap(), 5);
        assert_eq!(stream.read_long_le().unwrap(), 0x0102030405060708);
        assert!(stream.exhausted());
    }

    #[test]
    fn searches_remaining_bytes() {
        let mut stream = RealBufferedStream::from_bytes(b"Lorem ipsum".to_vec());
        stream.skip(1).unwrap();

        assert_eq!(stream.index_of_byte(b'm', 0), Some(3));
        assert_eq!(stream.index_of(b"ips", 0), Some(5));
        assert!(stream.range_equals(5, b"ipsum"));
    }

    #[test]
    fn peeks_without_consuming_source() {
        let mut stream = RealBufferedStream::from_bytes(b"abcdef".to_vec());
        let mut peek = stream.peek();

        assert_eq!(peek.read_byte_array(3).unwrap(), b"abc");
        assert_eq!(stream.read_byte_array(3).unwrap(), b"abc");
    }
}
