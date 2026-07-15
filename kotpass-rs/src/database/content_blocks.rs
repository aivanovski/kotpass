use cipher::KeyInit;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::{
    crypto::byte_array::{constant_time_equals, sha256, sha512},
    error::FormatError,
    io::buffered_stream::BufferedStream,
};

const BLOCK_SPLIT_RATE: usize = 1_048_576;
const HASH_SIZE: usize = 32;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Block<'a> {
    index: u64,
    length: usize,
    data: &'a [u8],
}

pub struct ContentBlocks;

impl ContentBlocks {
    pub fn read_content_blocks_ver3x<S: BufferedStream>(
        source: &mut S,
    ) -> Result<Vec<u8>, FormatError> {
        let mut content_data = Vec::new();

        loop {
            let index = read_i32_le(source)?;
            let hash = read_bytes(source, HASH_SIZE)?;
            let length = read_length(source)?;

            if length > 0 {
                let data = read_bytes(source, length)?;
                if sha256(&data) != hash {
                    return Err(FormatError::InvalidContent(format!(
                        "Hash for block {index} does not match."
                    )));
                }
                content_data.extend_from_slice(&data);
            } else {
                break;
            }
        }

        Ok(content_data)
    }

    pub fn read_content_blocks_ver4x<S: BufferedStream>(
        source: &mut S,
        master_seed: &[u8],
        transformed_key: &[u8],
    ) -> Result<Vec<u8>, FormatError> {
        let mut content_data = Vec::new();
        let hmac_key = create_block_hmac_key(master_seed, transformed_key);
        let mut index = 0;

        loop {
            let hash = read_bytes(source, HASH_SIZE)?;
            let length = read_length(source)?;

            if length > 0 {
                let data = read_bytes(source, length)?;
                if !constant_time_equals(&create_block_hmac(&hmac_key, index, length, &data), &hash)
                {
                    return Err(FormatError::InvalidContent(format!(
                        "HMAC for block {index} does not match."
                    )));
                }
                content_data.extend_from_slice(&data);
                index += 1;
            } else {
                break;
            }
        }

        Ok(content_data)
    }

    pub fn write_content_blocks_ver3x(content_data: &[u8]) -> Vec<u8> {
        write_content_blocks(content_data, true, |block| {
            if block.data.is_empty() {
                vec![0; HASH_SIZE]
            } else {
                sha256(block.data)
            }
        })
    }

    pub fn write_content_blocks_ver4x(
        content_data: &[u8],
        master_seed: &[u8],
        transformed_key: &[u8],
    ) -> Vec<u8> {
        let hmac_key = create_block_hmac_key(master_seed, transformed_key);

        write_content_blocks(content_data, false, |block| {
            create_block_hmac(&hmac_key, block.index, block.length, block.data)
        })
    }
}

fn write_content_blocks(
    content_data: &[u8],
    write_indexes: bool,
    hash_func: impl Fn(Block<'_>) -> Vec<u8>,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut index = 0;
    let mut offset = 0;

    while offset < content_data.len() {
        let length = (content_data.len() - offset).min(BLOCK_SPLIT_RATE);
        let data = &content_data[offset..offset + length];
        let hash = hash_func(Block {
            index,
            length,
            data,
        });

        if write_indexes {
            bytes.extend_from_slice(&(index as i32).to_le_bytes());
        }
        bytes.extend_from_slice(&hash);
        bytes.extend_from_slice(&(length as i32).to_le_bytes());
        bytes.extend_from_slice(data);

        index += 1;
        offset += length;
    }

    let hash = hash_func(Block {
        index,
        length: 0,
        data: &[],
    });
    if write_indexes {
        bytes.extend_from_slice(&(index as i32).to_le_bytes());
    }
    bytes.extend_from_slice(&hash);
    bytes.extend_from_slice(&0i32.to_le_bytes());

    bytes
}

fn create_block_hmac_key(master_seed: &[u8], transformed_key: &[u8]) -> Vec<u8> {
    let mut combined = Vec::with_capacity(master_seed.len() + transformed_key.len() + 1);
    combined.extend_from_slice(master_seed);
    combined.extend_from_slice(transformed_key);
    combined.push(0x01);
    sha512(&combined)
}

fn create_block_hmac(hmac_key: &[u8], index: u64, length: usize, data: &[u8]) -> Vec<u8> {
    let index_bytes = index.to_le_bytes();
    let mut block_key_source = Vec::with_capacity(index_bytes.len() + hmac_key.len());
    block_key_source.extend_from_slice(&index_bytes);
    block_key_source.extend_from_slice(hmac_key);
    let block_key = sha512(&block_key_source);

    let mut mac = HmacSha256::new_from_slice(&block_key).expect("HMAC accepts any key size");
    mac.update(&index_bytes);
    mac.update(&(length as i32).to_le_bytes());
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
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
            "Invalid content block length: {length}."
        )))
    } else {
        Ok(length as usize)
    }
}

fn read_bytes<S: BufferedStream>(source: &mut S, length: usize) -> Result<Vec<u8>, FormatError> {
    source
        .read_byte_array(length)
        .map_err(|error| FormatError::InvalidContent(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{BLOCK_SPLIT_RATE, ContentBlocks};
    use crate::{error::FormatError, io::real_buffered_stream::RealBufferedStream};

    const TEST_SENTENCE: &str = "Fusce gravida egestas scelerisque.";
    const TEST_DATA: &str = concat!(
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ",
        "Fusce gravida egestas scelerisque.",
        " Donec sem massa, laoreet ut mi at, placerat varius nulla."
    );

    #[test]
    fn reads_and_writes_sha_content_blocks() {
        let bytes = ContentBlocks::write_content_blocks_ver3x(TEST_DATA.as_bytes());
        let mut source = RealBufferedStream::from_bytes(bytes);
        let output = ContentBlocks::read_content_blocks_ver3x(&mut source).unwrap();
        let output = String::from_utf8(output).unwrap();

        assert!(output.contains(TEST_SENTENCE));
    }

    #[test]
    fn reads_and_writes_hmac_content_blocks() {
        let seed = [1; 32];
        let key = [2; 32];
        let bytes = ContentBlocks::write_content_blocks_ver4x(TEST_DATA.as_bytes(), &seed, &key);
        let mut source = RealBufferedStream::from_bytes(bytes);
        let output = ContentBlocks::read_content_blocks_ver4x(&mut source, &seed, &key).unwrap();
        let output = String::from_utf8(output).unwrap();

        assert!(output.contains(TEST_SENTENCE));
    }

    #[test]
    fn splits_large_v3_content_and_preserves_output() {
        let content = vec![7; BLOCK_SPLIT_RATE + 17];
        let bytes = ContentBlocks::write_content_blocks_ver3x(&content);
        let mut source = RealBufferedStream::from_bytes(bytes);

        assert_eq!(
            ContentBlocks::read_content_blocks_ver3x(&mut source).unwrap(),
            content
        );
    }

    #[test]
    fn rejects_v3_hash_mismatch() {
        let mut bytes = ContentBlocks::write_content_blocks_ver3x(TEST_DATA.as_bytes());
        bytes[4] ^= 0xff;
        let mut source = RealBufferedStream::from_bytes(bytes);

        let error = ContentBlocks::read_content_blocks_ver3x(&mut source).unwrap_err();

        assert_eq!(
            error,
            FormatError::InvalidContent("Hash for block 0 does not match.".to_owned())
        );
    }

    #[test]
    fn rejects_v4_hmac_mismatch() {
        let seed = [1; 32];
        let key = [2; 32];
        let bytes = ContentBlocks::write_content_blocks_ver4x(TEST_DATA.as_bytes(), &seed, &key);
        let mut source = RealBufferedStream::from_bytes(bytes);

        let error =
            ContentBlocks::read_content_blocks_ver4x(&mut source, &seed, &[3; 32]).unwrap_err();

        assert_eq!(
            error,
            FormatError::InvalidContent("HMAC for block 0 does not match.".to_owned())
        );
    }
}
