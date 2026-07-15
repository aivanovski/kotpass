use std::io::{self, ErrorKind};

use super::real_buffered_stream::RealBufferedStream;

pub trait BufferedStream {
    fn remaining(&self) -> &[u8];

    fn advance(&mut self, byte_count: usize);

    fn record_read(&mut self, _bytes: &[u8]) {}

    fn exhausted(&self) -> bool {
        self.remaining().is_empty()
    }

    fn request(&self, byte_count: usize) -> bool {
        self.remaining().len() >= byte_count
    }

    fn require(&self, byte_count: usize) -> io::Result<()> {
        if self.request(byte_count) {
            Ok(())
        } else {
            Err(io::Error::new(ErrorKind::UnexpectedEof, "not enough bytes"))
        }
    }

    fn read_byte(&mut self) -> io::Result<u8> {
        let bytes = self.read_byte_array(1)?;
        Ok(bytes[0])
    }

    fn read_short(&mut self) -> io::Result<i16> {
        Ok(i16::from_be_bytes(self.read_array()?))
    }

    fn read_short_le(&mut self) -> io::Result<i16> {
        Ok(i16::from_le_bytes(self.read_array()?))
    }

    fn read_int(&mut self) -> io::Result<i32> {
        Ok(i32::from_be_bytes(self.read_array()?))
    }

    fn read_int_le(&mut self) -> io::Result<i32> {
        Ok(i32::from_le_bytes(self.read_array()?))
    }

    fn read_long(&mut self) -> io::Result<i64> {
        Ok(i64::from_be_bytes(self.read_array()?))
    }

    fn read_long_le(&mut self) -> io::Result<i64> {
        Ok(i64::from_le_bytes(self.read_array()?))
    }

    fn read_byte_string(&mut self) -> Vec<u8> {
        self.read_byte_array(self.remaining().len())
            .expect("remaining length is available")
    }

    fn read_byte_string_len(&mut self, byte_count: usize) -> io::Result<Vec<u8>> {
        self.read_byte_array(byte_count)
    }

    fn read_byte_array_all(&mut self) -> Vec<u8> {
        self.read_byte_string()
    }

    fn read_byte_array(&mut self, byte_count: usize) -> io::Result<Vec<u8>> {
        self.require(byte_count)?;
        let bytes = self.remaining()[..byte_count].to_vec();
        self.advance(byte_count);
        self.record_read(&bytes);
        Ok(bytes)
    }

    fn read_fully(&mut self, sink: &mut [u8]) -> io::Result<()> {
        let bytes = self.read_byte_array(sink.len())?;
        sink.copy_from_slice(&bytes);
        Ok(())
    }

    fn read_into(&mut self, sink: &mut [u8]) -> io::Result<usize> {
        if sink.is_empty() {
            return Ok(0);
        }
        if self.exhausted() {
            return Ok(0);
        }

        let byte_count = sink.len().min(self.remaining().len());
        let bytes = self.remaining()[..byte_count].to_vec();
        sink[..byte_count].copy_from_slice(&bytes);
        self.advance(byte_count);
        self.record_read(&bytes);
        Ok(byte_count)
    }

    fn read_fully_to_vec(&mut self, sink: &mut Vec<u8>, byte_count: usize) -> io::Result<()> {
        sink.extend_from_slice(&self.read_byte_array(byte_count)?);
        Ok(())
    }

    fn read_all_to_vec(&mut self, sink: &mut Vec<u8>) -> usize {
        let bytes = self.read_byte_array_all();
        let byte_count = bytes.len();
        sink.extend_from_slice(&bytes);
        byte_count
    }

    fn index_of_byte(&self, byte: u8, from_index: usize) -> Option<usize> {
        self.remaining()
            .get(from_index..)?
            .iter()
            .position(|candidate| *candidate == byte)
            .map(|index| index + from_index)
    }

    fn index_of(&self, bytes: &[u8], from_index: usize) -> Option<usize> {
        if bytes.is_empty() {
            return Some(from_index.min(self.remaining().len()));
        }

        self.remaining()
            .get(from_index..)?
            .windows(bytes.len())
            .position(|candidate| candidate == bytes)
            .map(|index| index + from_index)
    }

    fn range_equals(&self, offset: usize, bytes: &[u8]) -> bool {
        self.remaining()
            .get(offset..offset.saturating_add(bytes.len()))
            == Some(bytes)
    }

    fn read_string(&mut self) -> io::Result<String> {
        bytes_to_string(self.read_byte_array_all())
    }

    fn read_string_len(&mut self, byte_count: usize) -> io::Result<String> {
        bytes_to_string(self.read_byte_array(byte_count)?)
    }

    fn skip(&mut self, byte_count: usize) -> io::Result<()> {
        let bytes = self.read_byte_array(byte_count)?;
        drop(bytes);
        Ok(())
    }

    fn peek(&self) -> RealBufferedStream {
        RealBufferedStream::from_bytes(self.remaining().to_vec())
    }

    fn read_array<const N: usize>(&mut self) -> io::Result<[u8; N]> {
        self.read_byte_array(N)?
            .try_into()
            .map_err(|_| io::Error::new(ErrorKind::UnexpectedEof, "not enough bytes"))
    }
}

fn bytes_to_string(bytes: Vec<u8>) -> io::Result<String> {
    String::from_utf8(bytes).map_err(|error| io::Error::new(ErrorKind::InvalidData, error))
}
