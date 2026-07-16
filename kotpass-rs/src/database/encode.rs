use std::io::{self, Write};

use cipher::KeyInit;
use flate2::{Compression as GzipCompression, write::GzEncoder};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use thiserror::Error;

use crate::{
    crypto::{
        BaseCipher, BaseKdfProvider, CipherProvider, EncryptionSaltGenerator, KdfProvider,
        KeyTransform,
        byte_array::{clear, sha256},
    },
    error::{CryptoError, FormatError},
    model::XmlEncodeContext,
    xml::{DEFAULT_XML_CONTENT_PARSER, XmlContentParser},
};

use super::{
    ContentBlocks, KeePassDatabase,
    header::{Compression, DatabaseHeader},
};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Error)]
pub enum DatabaseEncodeError {
    #[error(transparent)]
    Format(#[from] FormatError),

    #[error(transparent)]
    Crypto(#[from] CryptoError),

    #[error(transparent)]
    Io(#[from] io::Error),
}

impl KeePassDatabase {
    pub fn encode<W: Write>(&self, writer: W) -> Result<(), DatabaseEncodeError> {
        let base_ciphers = [BaseCipher::Aes, BaseCipher::ChaCha20];
        let cipher_providers: Vec<&dyn CipherProvider> = base_ciphers
            .iter()
            .map(|cipher| cipher as &dyn CipherProvider)
            .collect();

        self.encode_with(
            writer,
            &DEFAULT_XML_CONTENT_PARSER,
            &cipher_providers,
            &BaseKdfProvider,
        )
    }

    pub fn encode_with<W, P>(
        &self,
        mut writer: W,
        content_parser: &P,
        cipher_providers: &[&dyn CipherProvider],
        kdf_provider: &impl KdfProvider,
    ) -> Result<(), DatabaseEncodeError>
    where
        W: Write,
        P: XmlContentParser,
    {
        let mut transformed_key = transformed_key(kdf_provider, self.header(), self.credentials())?;
        let result = (|| {
            let mut header_bytes = self.header().write_to_bytes();
            let header_hash = sha256(&header_bytes);
            let mut raw_content = self.raw_content(content_parser, &header_hash)?;

            if self.header().compression() == Compression::GZip {
                raw_content = gzip(raw_content)?;
            }

            if matches!(self, Self::Ver4x { .. }) {
                let hmac_key =
                    KeyTransform::hmac_key(self.header().master_seed(), &transformed_key);
                let hmac_sha256 = create_hmac_sha256(&hmac_key, &header_bytes);
                header_bytes.extend_from_slice(&header_hash);
                header_bytes.extend_from_slice(&hmac_sha256);
            }

            writer.write_all(&header_bytes)?;
            let encrypted_content = encrypted_content(
                self.header(),
                &raw_content,
                &transformed_key,
                cipher_providers,
            )?;
            writer.write_all(&encrypted_content)?;
            Ok(())
        })();
        clear(&mut transformed_key);
        result
    }

    pub fn encode_as_xml(&self) -> Result<String, DatabaseEncodeError> {
        self.encode_as_xml_with(&DEFAULT_XML_CONTENT_PARSER)
    }

    pub fn encode_as_xml_with<P: XmlContentParser>(
        &self,
        content_parser: &P,
    ) -> Result<String, DatabaseEncodeError> {
        let mut context: XmlEncodeContext<EncryptionSaltGenerator> = XmlEncodeContext::Plain {
            version: self.header().version(),
            binaries: self.binaries().clone(),
            memory_protection_flags: self.content().meta.memory_protection.clone(),
        };
        Ok(content_parser.marshal_content(&mut context, self.content(), true)?)
    }

    fn raw_content<P: XmlContentParser>(
        &self,
        content_parser: &P,
        header_hash: &[u8],
    ) -> Result<Vec<u8>, DatabaseEncodeError> {
        match self {
            Self::Ver3x {
                header, content, ..
            } => {
                let DatabaseHeader::Ver3x {
                    version,
                    inner_random_stream_id,
                    inner_random_stream_key,
                    ..
                } = header
                else {
                    unreachable!("v3 database must contain v3 header")
                };
                let inner_encryption = EncryptionSaltGenerator::create(
                    *inner_random_stream_id,
                    inner_random_stream_key,
                )?;
                let mut context = XmlEncodeContext::Encrypted {
                    version: *version,
                    binaries: self.binaries().clone(),
                    inner_encryption,
                };
                let mut content = content.clone();
                content.meta.header_hash = Some(header_hash.to_vec());
                Ok(content_parser
                    .marshal_content(&mut context, &content, false)?
                    .into_bytes())
            }
            Self::Ver4x {
                header,
                content,
                inner_header,
                ..
            } => {
                let inner_encryption = EncryptionSaltGenerator::create(
                    inner_header.random_stream_id,
                    &inner_header.random_stream_key,
                )?;
                let mut context = XmlEncodeContext::Encrypted {
                    version: header.version(),
                    binaries: self.binaries().clone(),
                    inner_encryption,
                };
                let xml = content_parser
                    .marshal_content(&mut context, content, false)?
                    .into_bytes();
                let mut raw_content = inner_header.write_to_bytes()?;
                raw_content.extend_from_slice(&xml);
                Ok(raw_content)
            }
        }
    }

    fn binaries(&self) -> &indexmap::IndexMap<Vec<u8>, crate::model::BinaryData> {
        match self {
            Self::Ver3x { content, .. } => &content.meta.binaries,
            Self::Ver4x { inner_header, .. } => &inner_header.binaries,
        }
    }
}

fn transformed_key(
    kdf_provider: &impl KdfProvider,
    header: &DatabaseHeader,
    credentials: &super::Credentials,
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

fn encrypted_content(
    header: &DatabaseHeader,
    raw_content: &[u8],
    transformed_key: &[u8],
    cipher_providers: &[&dyn CipherProvider],
) -> Result<Vec<u8>, DatabaseEncodeError> {
    let cipher = cipher_providers
        .iter()
        .find(|provider| provider.uuid() == header.cipher_id())
        .ok_or_else(|| {
            FormatError::InvalidHeader(format!("Unsupported cipher ID ({}).", header.cipher_id()))
        })?;
    let master_key = KeyTransform::master_key(header.master_seed(), transformed_key);

    match header {
        DatabaseHeader::Ver3x {
            encryption_iv,
            stream_start_bytes,
            ..
        } => {
            let mut content = stream_start_bytes.clone();
            content.extend_from_slice(&ContentBlocks::write_content_blocks_ver3x(raw_content));
            Ok(cipher.encrypt(&master_key, encryption_iv, &content)?)
        }
        DatabaseHeader::Ver4x { encryption_iv, .. } => {
            let encrypted = cipher.encrypt(&master_key, encryption_iv, raw_content)?;
            Ok(ContentBlocks::write_content_blocks_ver4x(
                &encrypted,
                header.master_seed(),
                transformed_key,
            ))
        }
    }
}

fn gzip(data: Vec<u8>) -> Result<Vec<u8>, FormatError> {
    let mut encoder = GzEncoder::new(Vec::new(), GzipCompression::default());
    encoder
        .write_all(&data)
        .map_err(|error| FormatError::FailedCompression(error.to_string()))?;
    encoder
        .finish()
        .map_err(|error| FormatError::FailedCompression(error.to_string()))
}

fn create_hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::{
        crypto::EncryptedValue,
        database::{Credentials, KeePassDatabase},
    };

    const VER3_AES: &[u8] =
        include_bytes!("../../../kotpass/kotpass/src/test/resources/ver3_aes.kdbx");
    const VER4_AES: &[u8] =
        include_bytes!("../../../kotpass/kotpass/src/test/resources/ver4_aes.kdbx");

    fn credentials(passphrase: &str) -> Credentials {
        Credentials::from_passphrase(&EncryptedValue::from_string(passphrase).unwrap()).unwrap()
    }

    #[test]
    fn encodes_v3_database_that_can_be_decoded() {
        let database = KeePassDatabase::decode(Cursor::new(VER3_AES), credentials("1")).unwrap();
        let mut encoded = Vec::new();

        database.encode(&mut encoded).unwrap();
        let decoded = KeePassDatabase::decode(Cursor::new(encoded), credentials("1")).unwrap();

        assert!(matches!(decoded, KeePassDatabase::Ver3x { .. }));
        assert_eq!(decoded.content().group.name, database.content().group.name);
    }

    #[test]
    fn encodes_v4_database_that_can_be_decoded() {
        let database = KeePassDatabase::decode(Cursor::new(VER4_AES), credentials("1")).unwrap();
        let mut encoded = Vec::new();

        database.encode(&mut encoded).unwrap();
        let decoded = KeePassDatabase::decode(Cursor::new(encoded), credentials("1")).unwrap();

        assert!(matches!(decoded, KeePassDatabase::Ver4x { .. }));
        assert_eq!(decoded.content().group.name, database.content().group.name);
    }

    #[test]
    fn encodes_database_as_plain_xml() {
        let database =
            KeePassDatabase::create_ver4x("Root", crate::model::Meta::default(), credentials("1"))
                .unwrap();

        let xml = database.encode_as_xml().unwrap();

        assert!(xml.contains("<KeePassFile>"));
        assert!(xml.contains("<Name>Root</Name>"));
    }
}
