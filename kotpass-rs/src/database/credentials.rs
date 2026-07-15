use thiserror::Error;

use crate::{
    crypto::{EncryptedValue, byte_array::sha256},
    error::KeyfileError,
    io::{
        base16::{HexError, decode_hex_to_array, encode_hex},
        base64::{Base64Error, decode_base64_to_array},
    },
    xml::{
        Element, Node, PrintOptions, XmlParseError, XmlVersion,
        keyfile_xml::{attributes, tags},
        parse_str, xml,
    },
};

const XML_ENCODING: &str = "utf-8";
const DEFAULT_VERSION: &str = "2.0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Credentials {
    pub passphrase: Option<EncryptedValue>,
    pub key: Option<EncryptedValue>,
}

impl Credentials {
    pub fn from_passphrase(passphrase: &EncryptedValue) -> Result<Self, CredentialsError> {
        Ok(Self {
            passphrase: Some(EncryptedValue::from_binary(passphrase.get_hash())?),
            key: None,
        })
    }

    pub fn from_key_data(key_data: &[u8]) -> Result<Self, CredentialsError> {
        Ok(Self {
            passphrase: None,
            key: Some(EncryptedValue::from_binary(parse_keyfile(key_data)?)?),
        })
    }

    pub fn from_passphrase_and_key_data(
        passphrase: &EncryptedValue,
        key_data: &[u8],
    ) -> Result<Self, CredentialsError> {
        Ok(Self {
            passphrase: Some(EncryptedValue::from_binary(passphrase.get_hash())?),
            key: Some(EncryptedValue::from_binary(parse_keyfile(key_data)?)?),
        })
    }

    pub fn create_keyfile(key: &[u8]) -> String {
        let hash = encode_hex(&sha256(key)[..4]).to_uppercase();
        let key_hex = encode_hex(key).to_uppercase();
        let mut root = xml(
            tags::DOCUMENT,
            Some(XML_ENCODING),
            Some(XmlVersion::V10),
            None,
        );

        root.element(tags::META)
            .element_text(tags::VERSION, DEFAULT_VERSION);
        let data = root.element(tags::KEY).element(tags::DATA);
        data.attribute(attributes::HASH, hash);
        data.text(key_hex);

        root.to_string_with_options(PrintOptions {
            single_line_text_elements: true,
            ..PrintOptions::default()
        })
    }

    pub fn composite_key(&self) -> Vec<u8> {
        let mut composite = Vec::new();
        if let Some(passphrase) = &self.passphrase {
            composite.extend_from_slice(&passphrase.get_binary());
        }
        if let Some(key) = &self.key {
            composite.extend_from_slice(&key.get_binary());
        }
        sha256(&composite)
    }
}

#[derive(Debug, Error)]
pub enum CredentialsError {
    #[error(transparent)]
    Keyfile(#[from] KeyfileError),

    #[error(transparent)]
    Hex(#[from] HexError),

    #[error(transparent)]
    Base64(#[from] Base64Error),

    #[error(transparent)]
    Xml(#[from] XmlParseError),

    #[error(transparent)]
    Random(#[from] rand::rngs::SysError),
}

fn parse_keyfile(key_data: &[u8]) -> Result<Vec<u8>, CredentialsError> {
    match key_data.len() {
        32 => Ok(key_data.to_vec()),
        64 => {
            let text = String::from_utf8_lossy(key_data).to_lowercase();
            Ok(decode_hex_to_array(&text)?)
        }
        _ => match parse_xml_keyfile(key_data) {
            Ok(node) => find_xml_key_data(&node),
            Err(_) => Ok(sha256(key_data)),
        },
    }
}

fn parse_xml_keyfile(key_data: &[u8]) -> Result<Node, CredentialsError> {
    Ok(parse_str(
        std::str::from_utf8(key_data).map_err(|_| XmlParseError::MissingRoot)?,
    )?)
}

fn find_xml_key_data(node: &Node) -> Result<Vec<u8>, CredentialsError> {
    let version = node
        .first(tags::META)
        .and_then(|meta| meta.first(tags::VERSION))
        .and_then(text)
        .and_then(|value| value.parse::<f32>().ok())
        .ok_or(KeyfileError::InvalidVersion)?;
    let data_node = node
        .first(tags::KEY)
        .and_then(|key| key.first(tags::DATA))
        .ok_or(KeyfileError::NoKeyData)?;

    match version {
        version if version == 1.0 => text(data_node)
            .map(decode_base64_to_array)
            .ok_or(KeyfileError::NoKeyData)?
            .map_err(Into::into),
        version if version == 2.0 => {
            let hash = data_node
                .attribute_value(attributes::HASH)
                .map(decode_hex_to_array)
                .ok_or(KeyfileError::InvalidHash)?
                .map_err(CredentialsError::from)?;
            let data = text(data_node)
                .map(remove_whitespace)
                .map(|value| decode_hex_to_array(&value))
                .ok_or(KeyfileError::NoKeyData)?
                .map_err(CredentialsError::from)?;

            if sha256(&data)[..4] != hash {
                return Err(KeyfileError::InvalidHash.into());
            }

            Ok(data)
        }
        _ => Err(KeyfileError::InvalidVersion.into()),
    }
}

fn text(node: &Node) -> Option<&str> {
    node.children().first().and_then(|element| match element {
        Element::Text(text) => Some(text.text.as_str()),
        _ => None,
    })
}

fn remove_whitespace(value: &str) -> String {
    value.chars().filter(|char| !char.is_whitespace()).collect()
}

#[cfg(test)]
mod tests {
    use super::{Credentials, CredentialsError};
    use crate::{
        crypto::EncryptedValue,
        error::KeyfileError,
        io::{base16::encode_hex, base64::encode_base64},
    };

    const XML_KEYFILE_VER1: &str = r#"
        <KeyFile>
            <Meta>
                <Version>1.0</Version>
            </Meta>
            <Key>
                <Data>AtY2GR2pVt6aWz2ugfxfSQWjRId9l0JWe/LEMJWVJ1k=</Data>
            </Key>
        </KeyFile>
    "#;

    #[test]
    fn reads_from_xml_v1_keyfile() {
        let credentials = Credentials::from_key_data(XML_KEYFILE_VER1.as_bytes()).unwrap();

        assert_eq!(
            encode_hex(&credentials.composite_key()),
            "829bd09b8d05fafaa0e80b7307a978c496931815feb0a5cf82ce872ee36fa355"
        );
    }

    #[test]
    fn creates_xml_v2_keyfile() {
        let key = vec![1; 32];
        let keyfile = Credentials::create_keyfile(&key);
        let credentials = Credentials::from_key_data(keyfile.as_bytes()).unwrap();

        assert_eq!(credentials.key.unwrap().get_binary(), key);
    }

    #[test]
    fn reads_xml_v2_keyfile_with_whitespace_in_data() {
        let key = vec![2; 32];
        let keyfile = Credentials::create_keyfile(&key);
        let keyfile = keyfile.replace(
            &encode_hex(&key).to_uppercase(),
            &format!("{}\n{}", &encode_hex(&key)[..32], &encode_hex(&key)[32..]).to_uppercase(),
        );
        let credentials = Credentials::from_key_data(keyfile.as_bytes()).unwrap();

        assert_eq!(credentials.key.unwrap().get_binary(), key);
    }

    #[test]
    fn rejects_xml_v2_keyfile_with_invalid_hash() {
        let keyfile =
            Credentials::create_keyfile(&[1; 32]).replace("Hash=\"72CD6E84\"", "Hash=\"00000000\"");

        let error = Credentials::from_key_data(keyfile.as_bytes()).unwrap_err();

        assert!(matches!(
            error,
            CredentialsError::Keyfile(KeyfileError::InvalidHash)
        ));
    }

    #[test]
    fn reads_raw_32_byte_keyfile() {
        let key = vec![3; 32];
        let credentials = Credentials::from_key_data(&key).unwrap();

        assert_eq!(credentials.key.unwrap().get_binary(), key);
    }

    #[test]
    fn reads_raw_64_char_hex_keyfile() {
        let key = vec![4; 32];
        let credentials = Credentials::from_key_data(encode_hex(&key).as_bytes()).unwrap();

        assert_eq!(credentials.key.unwrap().get_binary(), key);
    }

    #[test]
    fn hashes_raw_binary_keyfile_when_xml_parse_fails() {
        let credentials = Credentials::from_key_data(b"not xml and not a fixed key").unwrap();

        assert_eq!(
            credentials.key.unwrap().get_binary(),
            crate::crypto::byte_array::sha256(b"not xml and not a fixed key")
        );
    }

    #[test]
    fn combines_hashed_passphrase_and_keyfile_in_composite_key() {
        let passphrase = EncryptedValue::from_string("secret").unwrap();
        let key = vec![5; 32];
        let credentials = Credentials::from_passphrase_and_key_data(&passphrase, &key).unwrap();
        let mut expected_input = passphrase.get_hash();
        expected_input.extend_from_slice(&key);

        assert_eq!(
            credentials.composite_key(),
            crate::crypto::byte_array::sha256(&expected_input)
        );
    }

    #[test]
    fn reads_xml_v1_base64_keyfile_data() {
        let key = vec![6; 32];
        let keyfile = format!(
            "<KeyFile><Meta><Version>1.0</Version></Meta><Key><Data>{}</Data></Key></KeyFile>",
            encode_base64(&key)
        );
        let credentials = Credentials::from_key_data(keyfile.as_bytes()).unwrap();

        assert_eq!(credentials.key.unwrap().get_binary(), key);
    }
}
