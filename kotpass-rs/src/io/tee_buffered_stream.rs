use std::{
    cell::RefCell,
    io::{self, Read},
    rc::Rc,
};

use super::{buffered_stream::BufferedStream, real_buffered_stream::RealBufferedStream};

pub type MirrorBuffer = Rc<RefCell<Vec<u8>>>;

#[derive(Debug, Clone)]
pub struct TeeBufferedStream {
    inner: RealBufferedStream,
    mirror: MirrorBuffer,
}

impl TeeBufferedStream {
    pub fn from_bytes(bytes: Vec<u8>, mirror: MirrorBuffer) -> Self {
        Self {
            inner: RealBufferedStream::from_bytes(bytes),
            mirror,
        }
    }

    pub fn from_reader<R: Read>(reader: R, mirror: MirrorBuffer) -> io::Result<Self> {
        Ok(Self {
            inner: RealBufferedStream::from_reader(reader)?,
            mirror,
        })
    }

    pub fn mirror(&self) -> MirrorBuffer {
        Rc::clone(&self.mirror)
    }
}

impl BufferedStream for TeeBufferedStream {
    fn remaining(&self) -> &[u8] {
        self.inner.remaining()
    }

    fn advance(&mut self, byte_count: usize) {
        self.inner.advance(byte_count);
    }

    fn record_read(&mut self, bytes: &[u8]) {
        self.mirror.borrow_mut().extend_from_slice(bytes);
    }
}

impl Read for TeeBufferedStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        BufferedStream::read_into(self, buf)
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::TeeBufferedStream;
    use crate::io::buffered_stream::BufferedStream;

    const DUMMY_DATA: &[u8] = b"Lorem ipsum";

    #[test]
    fn writes_read_data_to_mirror_buffer() {
        let mirror = Rc::new(RefCell::new(Vec::new()));
        let mut tee = TeeBufferedStream::from_bytes(DUMMY_DATA.to_vec(), Rc::clone(&mirror));

        let mut first = Vec::new();
        tee.read_fully_to_vec(&mut first, 5).unwrap();
        assert_eq!(&*mirror.borrow(), &DUMMY_DATA[..5]);

        let mut one = [0; 1];
        tee.read_into(&mut one).unwrap();
        assert_eq!(&*mirror.borrow(), &DUMMY_DATA[..6]);

        let mut rest = vec![0; DUMMY_DATA.len() - mirror.borrow().len()];
        tee.read_fully(&mut rest).unwrap();
        assert_eq!(&*mirror.borrow(), DUMMY_DATA);
    }

    #[test]
    fn does_not_write_to_mirror_while_peeking() {
        let mirror = Rc::new(RefCell::new(Vec::new()));
        let tee = TeeBufferedStream::from_bytes(DUMMY_DATA.to_vec(), Rc::clone(&mirror));
        let mut peek = tee.peek();

        assert_eq!(peek.read_byte_array(5).unwrap(), &DUMMY_DATA[..5]);
        assert!(mirror.borrow().is_empty());
    }
}
