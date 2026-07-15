use std::io;

use uuid::Uuid;

use crate::{
    constants::{CrsAlgorithm, HeaderFieldId},
    crypto::{BaseCipher, CipherProvider, secure_random::secure_random_bytes},
    error::FormatError,
    io::buffered_stream::BufferedStream,
    model::FormatVersion,
};

use super::{KdfParameters, Signature, VariantDictionary, VariantItems};

const END_OF_HEADER_BYTES: [u8; 4] = [0x0D, 0x0A, 0x0D, 0x0A];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Compression {
    None = 0,
    GZip = 1,
}

impl Compression {
    pub const ALL: [Self; 2] = [Self::None, Self::GZip];

    pub const fn ordinal(self) -> usize {
        self as usize
    }

    pub fn from_ordinal(ordinal: usize) -> Option<Self> {
        Self::ALL.get(ordinal).copied()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabaseHeader {
    Ver3x {
        signature: Signature,
        version: FormatVersion,
        cipher_id: Uuid,
        compression: Compression,
        master_seed: Vec<u8>,
        encryption_iv: Vec<u8>,
        transform_seed: Vec<u8>,
        transform_rounds: u64,
        inner_random_stream_id: CrsAlgorithm,
        inner_random_stream_key: Vec<u8>,
        stream_start_bytes: Vec<u8>,
    },
    Ver4x {
        signature: Signature,
        version: FormatVersion,
        cipher_id: Uuid,
        compression: Compression,
        master_seed: Vec<u8>,
        encryption_iv: Vec<u8>,
        kdf_parameters: KdfParameters,
        public_custom_data: VariantItems,
    },
}

impl DatabaseHeader {
    pub fn create_ver3x() -> Result<Self, FormatError> {
        Ok(Self::Ver3x {
            signature: Signature::DEFAULT,
            version: FormatVersion::new(3, 1),
            cipher_id: BaseCipher::Aes.uuid(),
            compression: Compression::GZip,
            master_seed: random_bytes(32)?,
            encryption_iv: random_bytes(BaseCipher::Aes.iv_len())?,
            transform_seed: random_bytes(32)?,
            transform_rounds: 6000,
            inner_random_stream_id: CrsAlgorithm::Salsa20,
            inner_random_stream_key: random_bytes(32)?,
            stream_start_bytes: random_bytes(32)?,
        })
    }

    pub fn create_ver4x() -> Result<Self, FormatError> {
        Ok(Self::Ver4x {
            signature: Signature::DEFAULT,
            version: FormatVersion::new(4, 1),
            cipher_id: BaseCipher::Aes.uuid(),
            compression: Compression::GZip,
            master_seed: random_bytes(32)?,
            encryption_iv: random_bytes(BaseCipher::Aes.iv_len())?,
            kdf_parameters: KdfParameters::argon2_default(random_bytes(32)?),
            public_custom_data: VariantItems::new(),
        })
    }

    pub const fn signature(&self) -> Signature {
        match self {
            Self::Ver3x { signature, .. } | Self::Ver4x { signature, .. } => *signature,
        }
    }

    pub const fn version(&self) -> FormatVersion {
        match self {
            Self::Ver3x { version, .. } | Self::Ver4x { version, .. } => *version,
        }
    }

    pub const fn cipher_id(&self) -> Uuid {
        match self {
            Self::Ver3x { cipher_id, .. } | Self::Ver4x { cipher_id, .. } => *cipher_id,
        }
    }

    pub const fn compression(&self) -> Compression {
        match self {
            Self::Ver3x { compression, .. } | Self::Ver4x { compression, .. } => *compression,
        }
    }

    pub fn master_seed(&self) -> &[u8] {
        match self {
            Self::Ver3x { master_seed, .. } | Self::Ver4x { master_seed, .. } => master_seed,
        }
    }

    pub fn encryption_iv(&self) -> &[u8] {
        match self {
            Self::Ver3x { encryption_iv, .. } | Self::Ver4x { encryption_iv, .. } => encryption_iv,
        }
    }

    pub fn read_from<S: BufferedStream>(source: &mut S) -> Result<Self, FormatError> {
        let signature = Signature::read_from(source).map_err(invalid_header)?;
        let version = FormatVersion::read_from(source).map_err(invalid_header)?;
        let mut fields = HeaderFields::default();

        loop {
            let (id, data) = read_header_value(source, version)?;
            let field_id = HeaderFieldId::from_id(id).ok_or_else(|| {
                FormatError::InvalidHeader("Unsupported header field ID.".to_owned())
            })?;

            match field_id {
                HeaderFieldId::EndOfHeader => break,
                HeaderFieldId::Comment => {}
                HeaderFieldId::CipherId => fields.cipher_id = Some(read_uuid(&data)?),
                HeaderFieldId::Compression => {
                    let ordinal = read_i32_le(&data)?;
                    fields.compression = Compression::from_ordinal(ordinal as usize);
                    if fields.compression.is_none() {
                        return Err(FormatError::InvalidHeader(format!(
                            "Unsupported compression algorithm: {ordinal}."
                        )));
                    }
                }
                HeaderFieldId::MasterSeed => fields.master_seed = Some(data),
                HeaderFieldId::TransformSeed => fields.transform_seed = Some(data),
                HeaderFieldId::TransformRounds => {
                    fields.transform_rounds = Some(read_i64_le(&data)? as u64);
                }
                HeaderFieldId::EncryptionIv => fields.encryption_iv = Some(data),
                HeaderFieldId::InnerRandomStreamKey => fields.inner_random_stream_key = Some(data),
                HeaderFieldId::StreamStartBytes => fields.stream_start_bytes = Some(data),
                HeaderFieldId::InnerRandomStreamId => {
                    let ordinal = read_i32_le(&data)?;
                    fields.inner_random_stream_id = CrsAlgorithm::from_ordinal(ordinal as usize);
                    if fields.inner_random_stream_id.is_none() {
                        return Err(FormatError::InvalidHeader(format!(
                            "Unsupported inner random stream ID: {ordinal}."
                        )));
                    }
                }
                HeaderFieldId::KdfParameters => {
                    fields.kdf_parameters = Some(KdfParameters::read_from(&data)?);
                }
                HeaderFieldId::PublicCustomData => {
                    fields.public_custom_data = VariantDictionary::read_from(&data)?;
                }
            }
        }

        if version.major < 4 {
            Ok(Self::Ver3x {
                signature,
                version,
                cipher_id: required(fields.cipher_id, "No cipher ID.")?,
                compression: required(fields.compression, "No compression.")?,
                master_seed: required(fields.master_seed, "No master seed.")?,
                encryption_iv: required(fields.encryption_iv, "No encryption IV.")?,
                transform_seed: required(fields.transform_seed, "No transform seed.")?,
                transform_rounds: required(fields.transform_rounds, "No transform rounds.")?,
                inner_random_stream_id: required(
                    fields.inner_random_stream_id,
                    "No inner random stream ID.",
                )?,
                inner_random_stream_key: required(
                    fields.inner_random_stream_key,
                    "No protected stream key.",
                )?,
                stream_start_bytes: required(fields.stream_start_bytes, "No stream start bytes.")?,
            })
        } else {
            Ok(Self::Ver4x {
                signature,
                version,
                cipher_id: required(fields.cipher_id, "No cipher ID.")?,
                compression: required(fields.compression, "No compression.")?,
                master_seed: required(fields.master_seed, "No master seed.")?,
                encryption_iv: required(fields.encryption_iv, "No encryption IV.")?,
                kdf_parameters: required(fields.kdf_parameters, "No kdf parameters found.")?,
                public_custom_data: fields.public_custom_data,
            })
        }
    }

    pub fn write_to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.signature()
            .write_to(&mut bytes)
            .expect("writing to Vec cannot fail");
        self.version()
            .write_to(&mut bytes)
            .expect("writing to Vec cannot fail");

        write_header_value(
            self,
            &mut bytes,
            HeaderFieldId::CipherId,
            self.cipher_id().as_bytes(),
        );
        write_header_value(
            self,
            &mut bytes,
            HeaderFieldId::Compression,
            &(self.compression().ordinal() as i32).to_le_bytes(),
        );
        write_header_value(
            self,
            &mut bytes,
            HeaderFieldId::MasterSeed,
            self.master_seed(),
        );
        write_header_value(
            self,
            &mut bytes,
            HeaderFieldId::EncryptionIv,
            self.encryption_iv(),
        );

        match self {
            Self::Ver3x {
                transform_seed,
                transform_rounds,
                inner_random_stream_id,
                inner_random_stream_key,
                stream_start_bytes,
                ..
            } => {
                write_header_value(
                    self,
                    &mut bytes,
                    HeaderFieldId::TransformSeed,
                    transform_seed,
                );
                write_header_value(
                    self,
                    &mut bytes,
                    HeaderFieldId::TransformRounds,
                    &(*transform_rounds as i64).to_le_bytes(),
                );
                write_header_value(
                    self,
                    &mut bytes,
                    HeaderFieldId::InnerRandomStreamId,
                    &(*inner_random_stream_id as i32).to_le_bytes(),
                );
                write_header_value(
                    self,
                    &mut bytes,
                    HeaderFieldId::InnerRandomStreamKey,
                    inner_random_stream_key,
                );
                write_header_value(
                    self,
                    &mut bytes,
                    HeaderFieldId::StreamStartBytes,
                    stream_start_bytes,
                );
            }
            Self::Ver4x {
                kdf_parameters,
                public_custom_data,
                ..
            } => {
                let params = kdf_parameters.write_to_bytes();
                write_header_value(self, &mut bytes, HeaderFieldId::KdfParameters, &params);
                let custom_data = VariantDictionary::write_to_bytes(public_custom_data);
                write_header_value(
                    self,
                    &mut bytes,
                    HeaderFieldId::PublicCustomData,
                    &custom_data,
                );
            }
        }

        write_header_value(
            self,
            &mut bytes,
            HeaderFieldId::EndOfHeader,
            &END_OF_HEADER_BYTES,
        );
        bytes
    }
}

#[derive(Debug, Default)]
struct HeaderFields {
    cipher_id: Option<Uuid>,
    compression: Option<Compression>,
    master_seed: Option<Vec<u8>>,
    transform_seed: Option<Vec<u8>>,
    transform_rounds: Option<u64>,
    encryption_iv: Option<Vec<u8>>,
    inner_random_stream_key: Option<Vec<u8>>,
    stream_start_bytes: Option<Vec<u8>>,
    inner_random_stream_id: Option<CrsAlgorithm>,
    kdf_parameters: Option<KdfParameters>,
    public_custom_data: VariantItems,
}

fn random_bytes(length: usize) -> Result<Vec<u8>, FormatError> {
    secure_random_bytes(length).map_err(|error| {
        FormatError::InvalidHeader(format!(
            "Failed to generate database header random data: {error}."
        ))
    })
}

fn write_header_value(
    header: &DatabaseHeader,
    bytes: &mut Vec<u8>,
    id: HeaderFieldId,
    data: &[u8],
) {
    bytes.push(id.id());

    if matches!(header, DatabaseHeader::Ver4x { .. }) {
        let length = i32::try_from(data.len()).expect("header value length exceeds i32");
        bytes.extend_from_slice(&length.to_le_bytes());
    } else {
        let length = i16::try_from(data.len()).expect("v3 header value length exceeds i16");
        bytes.extend_from_slice(&length.to_le_bytes());
    }

    bytes.extend_from_slice(data);
}

fn read_header_value<S: BufferedStream>(
    source: &mut S,
    version: FormatVersion,
) -> Result<(u8, Vec<u8>), FormatError> {
    let id = source.read_byte().map_err(invalid_header)?;
    let length = if version.major >= 4 {
        source.read_int_le().map_err(invalid_header)?
    } else {
        source.read_short_le().map_err(invalid_header)? as i32
    };

    if length < 0 {
        return Err(FormatError::InvalidHeader(format!(
            "Invalid header field length: {length}."
        )));
    }

    let data = source
        .read_byte_string_len(length as usize)
        .map_err(invalid_header)?;
    Ok((id, data))
}

fn read_uuid(data: &[u8]) -> Result<Uuid, FormatError> {
    let bytes: [u8; 16] = data
        .try_into()
        .map_err(|_| FormatError::InvalidHeader("Cipher ID header must be 16 bytes.".to_owned()))?;
    Ok(Uuid::from_bytes(bytes))
}

fn read_i32_le(data: &[u8]) -> Result<i32, FormatError> {
    let bytes: [u8; 4] = data.try_into().map_err(|_| {
        FormatError::InvalidHeader("Header integer field must be 4 bytes.".to_owned())
    })?;
    Ok(i32::from_le_bytes(bytes))
}

fn read_i64_le(data: &[u8]) -> Result<i64, FormatError> {
    let bytes: [u8; 8] = data
        .try_into()
        .map_err(|_| FormatError::InvalidHeader("Header long field must be 8 bytes.".to_owned()))?;
    Ok(i64::from_le_bytes(bytes))
}

fn required<T>(value: Option<T>, message: &str) -> Result<T, FormatError> {
    value.ok_or_else(|| FormatError::InvalidHeader(message.to_owned()))
}

fn invalid_header(error: io::Error) -> FormatError {
    FormatError::InvalidHeader(error.to_string())
}

#[cfg(test)]
mod tests {
    use crate::{
        database::header::{Compression, DatabaseHeader, KdfParameters, Signature},
        io::real_buffered_stream::RealBufferedStream,
        model::FormatVersion,
    };

    const VER3_AES: &[u8] =
        include_bytes!("../../../../kotpass/kotpass/src/test/resources/ver3_aes.kdbx");
    const VER4_AES: &[u8] =
        include_bytes!("../../../../kotpass/kotpass/src/test/resources/ver4_aes.kdbx");
    const VER4_ARGON2: &[u8] =
        include_bytes!("../../../../kotpass/kotpass/src/test/resources/ver4_argon2.kdbx");

    #[test]
    fn creates_default_headers() {
        let ver3 = DatabaseHeader::create_ver3x().unwrap();
        let ver4 = DatabaseHeader::create_ver4x().unwrap();

        assert!(matches!(
            ver3,
            DatabaseHeader::Ver3x {
                version: FormatVersion { major: 3, minor: 1 },
                compression: Compression::GZip,
                ..
            }
        ));
        assert!(matches!(
            ver4,
            DatabaseHeader::Ver4x {
                version: FormatVersion { major: 4, minor: 1 },
                compression: Compression::GZip,
                kdf_parameters: KdfParameters::Argon2 { .. },
                ..
            }
        ));
    }

    #[test]
    fn reads_kdf_parameters_from_kdbx_fixtures() {
        let mut source = RealBufferedStream::from_bytes(VER4_ARGON2.to_vec());
        let ver4_argon2 = DatabaseHeader::read_from(&mut source).unwrap();

        assert_eq!(ver4_argon2.signature().base, Signature::BASE);
        assert!(matches!(
            ver4_argon2,
            DatabaseHeader::Ver4x {
                kdf_parameters: KdfParameters::Argon2 { .. },
                ..
            }
        ));

        let mut source = RealBufferedStream::from_bytes(VER4_AES.to_vec());
        let ver4_aes = DatabaseHeader::read_from(&mut source).unwrap();
        assert!(matches!(
            ver4_aes,
            DatabaseHeader::Ver4x {
                kdf_parameters: KdfParameters::Aes { .. },
                ..
            }
        ));

        let mut source = RealBufferedStream::from_bytes(VER3_AES.to_vec());
        let ver3_aes = DatabaseHeader::read_from(&mut source).unwrap();
        assert!(matches!(ver3_aes, DatabaseHeader::Ver3x { .. }));
    }

    #[test]
    fn writes_and_reads_v4_header() {
        let mut source = RealBufferedStream::from_bytes(VER4_ARGON2.to_vec());
        let header = DatabaseHeader::read_from(&mut source).unwrap();
        let bytes = header.write_to_bytes();

        let mut source = RealBufferedStream::from_bytes(bytes);
        let header = DatabaseHeader::read_from(&mut source).unwrap();

        assert_eq!(header.signature().base, Signature::BASE);
        assert!(matches!(
            header,
            DatabaseHeader::Ver4x {
                kdf_parameters: KdfParameters::Argon2 { .. },
                ..
            }
        ));
    }
}
