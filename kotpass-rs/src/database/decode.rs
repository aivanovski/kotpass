use std::{
    cell::RefCell,
    io::{self, Read},
    rc::Rc,
};

use cipher::KeyInit;
use flate2::read::GzDecoder;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use thiserror::Error;

use crate::{
    constants::defaults,
    crypto::{
        BaseCipher, BaseKdfProvider, CipherProvider, EncryptionSaltGenerator, KdfProvider,
        KeyTransform,
        byte_array::{clear, constant_time_equals, sha256},
    },
    error::{CryptoError, FormatError},
    io::{
        buffered_stream::BufferedStream, real_buffered_stream::RealBufferedStream,
        tee_buffered_stream::TeeBufferedStream,
    },
    model::XmlDecodeContext,
    xml::{DEFAULT_XML_CONTENT_PARSER, XmlContentParser},
};

use super::{
    ContentBlocks, Credentials, KeePassDatabase, MAX_SUPPORTED_VERSION, MIN_SUPPORTED_VERSION,
    header::{Compression, DatabaseHeader, DatabaseInnerHeader, Signature},
};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Error)]
pub enum DatabaseDecodeError {
    #[error(transparent)]
    Format(#[from] FormatError),

    #[error(transparent)]
    Crypto(#[from] CryptoError),

    #[error(transparent)]
    Io(#[from] io::Error),
}

impl KeePassDatabase {
    pub fn decode<R: Read>(
        reader: R,
        credentials: Credentials,
    ) -> Result<Self, DatabaseDecodeError> {
        let base_ciphers = [BaseCipher::Aes, BaseCipher::ChaCha20];
        let cipher_providers: Vec<&dyn CipherProvider> = base_ciphers
            .iter()
            .map(|cipher| cipher as &dyn CipherProvider)
            .collect();

        Self::decode_with(
            reader,
            credentials,
            true,
            &DEFAULT_XML_CONTENT_PARSER,
            &cipher_providers,
            &BaseKdfProvider,
            defaults::UNTITLED_LABEL,
        )
    }

    pub fn decode_with<R, P>(
        reader: R,
        credentials: Credentials,
        validate_hashes: bool,
        content_parser: &P,
        cipher_providers: &[&dyn CipherProvider],
        kdf_provider: &impl KdfProvider,
        untitled_label: &str,
    ) -> Result<Self, DatabaseDecodeError>
    where
        R: Read,
        P: XmlContentParser,
    {
        let header_buffer = Rc::new(RefCell::new(Vec::new()));
        let mut source = TeeBufferedStream::from_reader(reader, Rc::clone(&header_buffer))?;
        let header = DatabaseHeader::read_from(&mut source)?;

        validate_header(&header)?;

        let raw_header_data = header_buffer.borrow().clone();
        let mut transformed_key = transformed_key(kdf_provider, &header, &credentials)?;
        let result = match &header {
            DatabaseHeader::Ver3x {
                version,
                inner_random_stream_id,
                inner_random_stream_key,
                ..
            } => {
                let salt_generator = EncryptionSaltGenerator::create(
                    *inner_random_stream_id,
                    inner_random_stream_key,
                )?;
                let raw_content =
                    decrypt_raw_content(&header, &mut source, &transformed_key, cipher_providers)?;
                let content = content_parser.unmarshal_content_bytes(&raw_content, |meta| {
                    XmlDecodeContext {
                        version: *version,
                        encryption: salt_generator,
                        binaries: meta.binaries.clone(),
                        untitled_label: untitled_label.to_owned(),
                    }
                })?;

                if validate_hashes {
                    if let Some(header_hash) = &content.meta.header_hash {
                        if header_hash != &sha256(&raw_header_data) {
                            return Err(FormatError::InvalidHeader(
                                "HeaderHash value does not match Sha256 of the header.".to_owned(),
                            )
                            .into());
                        }
                    }
                }

                Ok(Self::Ver3x {
                    credentials,
                    header,
                    content,
                })
            }
            DatabaseHeader::Ver4x { version, .. } => {
                let expected_sha256 = source
                    .read_byte_string_len(32)
                    .map_err(|error| FormatError::InvalidHeader(error.to_string()))?;
                let expected_hmac_sha256 = source
                    .read_byte_string_len(32)
                    .map_err(|error| FormatError::InvalidHeader(error.to_string()))?;

                if validate_hashes {
                    if !constant_time_equals(&sha256(&raw_header_data), &expected_sha256) {
                        return Err(FormatError::InvalidHeader(
                            "Header's Sha256 does not match.".to_owned(),
                        )
                        .into());
                    }

                    let hmac_key = KeyTransform::hmac_key(header.master_seed(), &transformed_key);
                    let hmac_sha256 = create_hmac_sha256(&hmac_key, &raw_header_data);
                    if !constant_time_equals(&hmac_sha256, &expected_hmac_sha256) {
                        return Err(CryptoError::InvalidKey(
                            "Wrong key used for decryption.".to_owned(),
                        )
                        .into());
                    }
                }

                let raw_content =
                    decrypt_raw_content(&header, &mut source, &transformed_key, cipher_providers)?;
                let mut raw_content = RealBufferedStream::from_bytes(raw_content);
                let inner_header = DatabaseInnerHeader::read_from(&mut raw_content)?;
                let salt_generator = EncryptionSaltGenerator::create(
                    inner_header.random_stream_id,
                    &inner_header.random_stream_key,
                )?;
                let xml_data = raw_content.read_byte_array_all();
                let content =
                    content_parser.unmarshal_content_bytes(&xml_data, |_| XmlDecodeContext {
                        version: *version,
                        encryption: salt_generator,
                        binaries: inner_header.binaries.clone(),
                        untitled_label: untitled_label.to_owned(),
                    })?;

                Ok(Self::Ver4x {
                    credentials,
                    header,
                    content,
                    inner_header,
                })
            }
        };
        clear(&mut transformed_key);
        result
    }

    pub fn decode_from_xml<R: Read>(
        reader: R,
        credentials: Credentials,
    ) -> Result<Self, DatabaseDecodeError> {
        Self::decode_from_xml_with(
            reader,
            credentials,
            &DEFAULT_XML_CONTENT_PARSER,
            defaults::UNTITLED_LABEL,
        )
    }

    pub fn decode_from_xml_with<R, P>(
        reader: R,
        credentials: Credentials,
        content_parser: &P,
        untitled_label: &str,
    ) -> Result<Self, DatabaseDecodeError>
    where
        R: Read,
        P: XmlContentParser,
    {
        let header = DatabaseHeader::create_ver4x()?;
        let mut inner_header = DatabaseInnerHeader::create()?;
        let salt_generator = EncryptionSaltGenerator::create(
            inner_header.random_stream_id,
            &inner_header.random_stream_key,
        )?;
        let mut content =
            content_parser.unmarshal_content_reader(reader, |meta| XmlDecodeContext {
                version: header.version(),
                encryption: salt_generator,
                binaries: meta.binaries.clone(),
                untitled_label: untitled_label.to_owned(),
            })?;

        inner_header.binaries = content.meta.binaries.clone();
        content.meta.binaries.clear();

        Ok(Self::Ver4x {
            credentials,
            header,
            content,
            inner_header,
        })
    }
}

fn validate_header(header: &DatabaseHeader) -> Result<(), FormatError> {
    if header.signature().base != Signature::BASE {
        return Err(FormatError::UnknownFormat(
            "File has unexpected signature.".to_owned(),
        ));
    }
    if header.signature().secondary != Signature::SECONDARY
        || header.version().major < MIN_SUPPORTED_VERSION as i16
        || header.version().major > MAX_SUPPORTED_VERSION as i16
    {
        return Err(FormatError::UnsupportedVersion(
            "File version is not supported.".to_owned(),
        ));
    }
    Ok(())
}

fn transformed_key(
    kdf_provider: &impl KdfProvider,
    header: &DatabaseHeader,
    credentials: &Credentials,
) -> Result<Vec<u8>, CryptoError> {
    let mut composite_key = credentials.composite_key();
    let result = match header {
        DatabaseHeader::Ver3x {
            transform_rounds,
            transform_seed,
            ..
        } => kdf_provider.transform_key(
            &crate::crypto::KdfParameters::Aes {
                rounds: *transform_rounds,
                seed: transform_seed.clone(),
            },
            &composite_key,
        ),
        DatabaseHeader::Ver4x { kdf_parameters, .. } => {
            kdf_provider.transform_key(&kdf_parameters.to_crypto_parameters(), &composite_key)
        }
    };
    clear(&mut composite_key);
    result
}

fn decrypt_raw_content<S: BufferedStream>(
    header: &DatabaseHeader,
    source: &mut S,
    transformed_key: &[u8],
    cipher_providers: &[&dyn CipherProvider],
) -> Result<Vec<u8>, DatabaseDecodeError> {
    let cipher = cipher_providers
        .iter()
        .find(|provider| provider.uuid() == header.cipher_id())
        .ok_or_else(|| {
            FormatError::InvalidHeader(format!("Unsupported cipher ID ({}).", header.cipher_id()))
        })?;
    let master_key = KeyTransform::master_key(header.master_seed(), transformed_key);

    let decrypted_content = match header {
        DatabaseHeader::Ver3x {
            encryption_iv,
            stream_start_bytes,
            ..
        } => {
            let encrypted_content = source.read_byte_array_all();
            let content_blocks = cipher.decrypt(&master_key, encryption_iv, &encrypted_content)?;
            if !content_blocks.starts_with(stream_start_bytes) {
                return Err(FormatError::InvalidContent(
                    "Database content could be corrupted or cannot be decrypted.".to_owned(),
                )
                .into());
            }
            let mut source =
                RealBufferedStream::from_bytes(content_blocks[stream_start_bytes.len()..].to_vec());
            ContentBlocks::read_content_blocks_ver3x(&mut source)?
        }
        DatabaseHeader::Ver4x { encryption_iv, .. } => {
            let encrypted_content = ContentBlocks::read_content_blocks_ver4x(
                source,
                header.master_seed(),
                transformed_key,
            )?;
            cipher.decrypt(&master_key, encryption_iv, &encrypted_content)?
        }
    };

    decompress(header.compression(), decrypted_content).map_err(Into::into)
}

fn decompress(compression: Compression, data: Vec<u8>) -> Result<Vec<u8>, FormatError> {
    match compression {
        Compression::None => Ok(data),
        Compression::GZip => {
            let mut decoder = GzDecoder::new(data.as_slice());
            let mut decompressed = Vec::new();
            decoder
                .read_to_end(&mut decompressed)
                .map_err(|error| FormatError::FailedCompression(error.to_string()))?;
            Ok(decompressed)
        }
    }
}

fn create_hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::DatabaseDecodeError;
    use crate::{
        crypto::{EncryptedValue, EncryptionSaltGenerator},
        database::{Credentials, KeePassDatabase},
        error::FormatError,
        model::XmlEncodeContext,
        xml::{DEFAULT_XML_CONTENT_PARSER, XmlContentParser},
    };

    const VER3_AES: &[u8] =
        include_bytes!("../../../kotpass/kotpass/src/test/resources/ver3_aes.kdbx");
    const VER4_AES: &[u8] =
        include_bytes!("../../../kotpass/kotpass/src/test/resources/ver4_aes.kdbx");

    fn credentials(passphrase: &str) -> Credentials {
        Credentials::from_passphrase(&EncryptedValue::from_string(passphrase).unwrap()).unwrap()
    }

    #[test]
    fn decodes_v3_aes_database() {
        let database = KeePassDatabase::decode(Cursor::new(VER3_AES), credentials("1")).unwrap();

        assert!(matches!(database, KeePassDatabase::Ver3x { .. }));
        assert!(!database.content().group.name.is_empty());
    }

    #[test]
    fn decodes_v4_aes_database() {
        let database = KeePassDatabase::decode(Cursor::new(VER4_AES), credentials("1")).unwrap();

        assert!(matches!(database, KeePassDatabase::Ver4x { .. }));
        assert!(!database.content().group.name.is_empty());
    }

    #[test]
    fn rejects_wrong_credentials() {
        let error =
            KeePassDatabase::decode(Cursor::new(VER4_AES), credentials("wrong")).unwrap_err();

        assert!(matches!(
            error,
            DatabaseDecodeError::Crypto(crate::error::CryptoError::InvalidKey(_))
                | DatabaseDecodeError::Format(FormatError::InvalidContent(_))
        ));
    }

    #[test]
    fn decodes_plain_xml_as_v4_database() {
        let source =
            KeePassDatabase::create_ver4x("Root", crate::model::Meta::default(), credentials("1"))
                .unwrap();
        let mut context: XmlEncodeContext<EncryptionSaltGenerator> = XmlEncodeContext::Plain {
            version: source.header().version(),
            binaries: source.content().meta.binaries.clone(),
            memory_protection_flags: source.content().meta.memory_protection.clone(),
        };
        let xml = DEFAULT_XML_CONTENT_PARSER
            .marshal_content(&mut context, source.content(), false)
            .unwrap();

        let database =
            KeePassDatabase::decode_from_xml(Cursor::new(xml), credentials("1")).unwrap();

        assert!(matches!(database, KeePassDatabase::Ver4x { .. }));
        assert_eq!(database.content().group.name, "Root");
    }
}
