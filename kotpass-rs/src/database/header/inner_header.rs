use indexmap::IndexMap;

use crate::{
    constants::CrsAlgorithm, crypto::secure_random::secure_random_bytes, error::FormatError,
    io::buffered_stream::BufferedStream, model::BinaryData,
};

const BINARY_FLAGS_SIZE: usize = 1;
const DEFAULT_RANDOM_STREAM_KEY_SIZE: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum InnerHeaderFieldId {
    Terminator = 0x00,
    StreamId = 0x01,
    StreamKey = 0x02,
    Binary = 0x03,
}

impl InnerHeaderFieldId {
    fn from_id(id: u8) -> Option<Self> {
        match id {
            0x00 => Some(Self::Terminator),
            0x01 => Some(Self::StreamId),
            0x02 => Some(Self::StreamKey),
            0x03 => Some(Self::Binary),
            _ => None,
        }
    }

    const fn id(self) -> u8 {
        self as u8
    }
}

pub type InnerBinaries = IndexMap<Vec<u8>, BinaryData>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseInnerHeader {
    pub random_stream_id: CrsAlgorithm,
    pub random_stream_key: Vec<u8>,
    pub binaries: InnerBinaries,
}

impl DatabaseInnerHeader {
    pub fn create() -> Result<Self, FormatError> {
        Ok(Self {
            random_stream_id: CrsAlgorithm::ChaCha20,
            random_stream_key: secure_random_bytes(DEFAULT_RANDOM_STREAM_KEY_SIZE).map_err(
                |error| {
                    FormatError::InvalidContent(format!(
                        "Failed to generate random stream key: {error}."
                    ))
                },
            )?,
            binaries: IndexMap::new(),
        })
    }

    pub fn read_from<S: BufferedStream>(source: &mut S) -> Result<Self, FormatError> {
        let mut binaries = IndexMap::new();
        let mut random_stream_id = None;
        let mut random_stream_key = None;

        loop {
            let id = read_u8(source)?;
            let length = read_length(source)?;
            let field_id = InnerHeaderFieldId::from_id(id).ok_or_else(|| {
                FormatError::InvalidContent(format!("Unknown inner header id: {id}."))
            })?;

            match field_id {
                InnerHeaderFieldId::Terminator => {
                    read_bytes(source, length)?;
                    break;
                }
                InnerHeaderFieldId::StreamId => {
                    require_length(field_id, length, size_of::<i32>())?;
                    let ordinal = read_i32_le(source)?;
                    random_stream_id = CrsAlgorithm::from_ordinal(ordinal as usize);
                    if random_stream_id.is_none() {
                        return Err(FormatError::InvalidContent(format!(
                            "Unknown random stream id: {ordinal}."
                        )));
                    }
                }
                InnerHeaderFieldId::StreamKey => {
                    random_stream_key = Some(read_bytes(source, length)?);
                }
                InnerHeaderFieldId::Binary => {
                    if length < BINARY_FLAGS_SIZE {
                        return Err(FormatError::InvalidContent(
                            "Inner header binary is missing flags.".to_owned(),
                        ));
                    }

                    let memory_protection = read_u8(source)? != 0;
                    let content = read_bytes(source, length - BINARY_FLAGS_SIZE)?;
                    let binary = BinaryData::uncompressed(memory_protection, content);
                    binaries.insert(binary.hash().to_vec(), binary);
                }
            }
        }

        Ok(Self {
            random_stream_id: random_stream_id.ok_or_else(|| {
                FormatError::InvalidContent("No random stream id found in inner header".to_owned())
            })?,
            random_stream_key: random_stream_key.ok_or_else(|| {
                FormatError::InvalidContent("No random stream key found in inner header".to_owned())
            })?,
            binaries,
        })
    }

    pub fn write_to_bytes(&self) -> Result<Vec<u8>, FormatError> {
        let mut bytes = Vec::new();

        write_field(
            &mut bytes,
            InnerHeaderFieldId::StreamId,
            &(self.random_stream_id.ordinal() as i32).to_le_bytes(),
        );
        write_field(
            &mut bytes,
            InnerHeaderFieldId::StreamKey,
            &self.random_stream_key,
        );

        for binary in self.binaries.values() {
            let content = binary.get_content()?;
            write_field_header(
                &mut bytes,
                InnerHeaderFieldId::Binary,
                content.len() + BINARY_FLAGS_SIZE,
            );
            bytes.push(u8::from(binary.memory_protection()));
            bytes.extend_from_slice(&content);
        }

        write_field(&mut bytes, InnerHeaderFieldId::Terminator, &[]);

        Ok(bytes)
    }
}

fn write_field(bytes: &mut Vec<u8>, field_id: InnerHeaderFieldId, data: &[u8]) {
    write_field_header(bytes, field_id, data.len());
    bytes.extend_from_slice(data);
}

fn write_field_header(bytes: &mut Vec<u8>, field_id: InnerHeaderFieldId, length: usize) {
    bytes.push(field_id.id());
    let length = i32::try_from(length).expect("inner header field length exceeds i32");
    bytes.extend_from_slice(&length.to_le_bytes());
}

fn require_length(
    field_id: InnerHeaderFieldId,
    actual: usize,
    expected: usize,
) -> Result<(), FormatError> {
    if actual == expected {
        Ok(())
    } else {
        Err(FormatError::InvalidContent(format!(
            "Invalid inner header field length for {:?}.",
            field_id
        )))
    }
}

fn read_u8<S: BufferedStream>(source: &mut S) -> Result<u8, FormatError> {
    source
        .read_byte()
        .map_err(|error| FormatError::InvalidContent(error.to_string()))
}

fn read_i32_le<S: BufferedStream>(source: &mut S) -> Result<i32, FormatError> {
    source
        .read_int_le()
        .map_err(|error| FormatError::InvalidContent(error.to_string()))
}

fn read_length<S: BufferedStream>(source: &mut S) -> Result<usize, FormatError> {
    let length = read_i32_le(source)?;
    if length < 0 {
        Err(FormatError::InvalidContent(format!(
            "Invalid inner header field length: {length}."
        )))
    } else {
        Ok(length as usize)
    }
}

fn read_bytes<S: BufferedStream>(source: &mut S, length: usize) -> Result<Vec<u8>, FormatError> {
    source
        .read_byte_string_len(length)
        .map_err(|error| FormatError::InvalidContent(error.to_string()))
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use super::DatabaseInnerHeader;
    use crate::{
        constants::CrsAlgorithm, io::real_buffered_stream::RealBufferedStream, model::BinaryData,
    };

    const INNER_HEADER_WITH_BINARIES: &[u8] =
        include_bytes!("../../../../kotpass/kotpass/src/test/resources/inner_header_with_binaries");

    #[test]
    fn creates_default_v4_inner_header() {
        let header = DatabaseInnerHeader::create().unwrap();

        assert_eq!(header.random_stream_id, CrsAlgorithm::ChaCha20);
        assert_eq!(header.random_stream_key.len(), 64);
        assert!(header.binaries.is_empty());
    }

    #[test]
    fn reads_and_writes_fixture_with_binaries() {
        let mut source = RealBufferedStream::from_bytes(INNER_HEADER_WITH_BINARIES.to_vec());
        let header = DatabaseInnerHeader::read_from(&mut source).unwrap();

        assert_eq!(header.random_stream_id, CrsAlgorithm::ChaCha20);
        assert_eq!(header.random_stream_key.len(), 64);
        assert_eq!(header.binaries.len(), 2);

        let bytes = header.write_to_bytes().unwrap();
        let mut source = RealBufferedStream::from_bytes(bytes);
        let header = DatabaseInnerHeader::read_from(&mut source).unwrap();

        assert_eq!(header.random_stream_id, CrsAlgorithm::ChaCha20);
        assert_eq!(header.random_stream_key.len(), 64);
        assert_eq!(header.binaries.len(), 2);
    }

    #[test]
    fn writes_binary_flags_and_hash_keys() {
        let first = BinaryData::uncompressed(true, b"first".to_vec());
        let second = BinaryData::uncompressed(false, b"second".to_vec());
        let mut binaries = IndexMap::new();
        binaries.insert(first.hash().to_vec(), first);
        binaries.insert(second.hash().to_vec(), second);
        let header = DatabaseInnerHeader {
            random_stream_id: CrsAlgorithm::Salsa20,
            random_stream_key: b"key".to_vec(),
            binaries,
        };

        let bytes = header.write_to_bytes().unwrap();
        let mut source = RealBufferedStream::from_bytes(bytes);
        let header = DatabaseInnerHeader::read_from(&mut source).unwrap();

        assert_eq!(header.random_stream_id, CrsAlgorithm::Salsa20);
        assert_eq!(header.random_stream_key, b"key");
        assert_eq!(header.binaries.len(), 2);
        assert!(header.binaries.values().next().unwrap().memory_protection());
        assert!(!header.binaries.values().nth(1).unwrap().memory_protection());
    }
}
