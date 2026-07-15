use std::io::{self, Read};

use super::{
    real_buffered_stream::RealBufferedStream,
    tee_buffered_stream::{MirrorBuffer, TeeBufferedStream},
};

pub fn buffer_stream<R: Read>(reader: R) -> io::Result<RealBufferedStream> {
    RealBufferedStream::from_reader(reader)
}

pub fn tee_buffer_stream<R: Read>(
    reader: R,
    mirror: MirrorBuffer,
) -> io::Result<TeeBufferedStream> {
    TeeBufferedStream::from_reader(reader, mirror)
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, io::Cursor, rc::Rc};

    use super::{buffer_stream, tee_buffer_stream};
    use crate::io::buffered_stream::BufferedStream;

    #[test]
    fn creates_real_buffered_stream_from_reader() {
        let mut stream = buffer_stream(Cursor::new(b"abc")).unwrap();

        assert_eq!(stream.read_byte_array(2).unwrap(), b"ab");
    }

    #[test]
    fn creates_tee_buffered_stream_from_reader() {
        let mirror = Rc::new(RefCell::new(Vec::new()));
        let mut stream = tee_buffer_stream(Cursor::new(b"abc"), Rc::clone(&mirror)).unwrap();

        assert_eq!(stream.read_byte_array(2).unwrap(), b"ab");
        assert_eq!(&*mirror.borrow(), b"ab");
    }
}
