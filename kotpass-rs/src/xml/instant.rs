use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::{
    io::base64::{Base64Error, decode_base64_to_array, encode_base64},
    model::XmlEncodeContext,
    xml::{Node, NodeXmlExt},
};

pub const EPOCH_SECONDS_FROM_AD: i64 = 62_135_596_800;

pub fn parse_instant(text: &str) -> Result<OffsetDateTime, XmlInstantError> {
    if text.find(':').is_some_and(|index| index > 0) {
        Ok(OffsetDateTime::parse(text, &Rfc3339)?)
    } else {
        let bytes = decode_base64_to_array(text)?;
        let length = bytes.len();
        let bytes: [u8; 8] = bytes
            .try_into()
            .map_err(|_| XmlInstantError::InvalidBinaryTimestampLength(length))?;
        let seconds = i64::from_le_bytes(bytes);
        Ok(OffsetDateTime::from_unix_timestamp(
            seconds - EPOCH_SECONDS_FROM_AD,
        )?)
    }
}

pub fn marshal_instant<E>(
    instant: OffsetDateTime,
    context: &XmlEncodeContext<E>,
) -> Result<String, XmlInstantError> {
    if context.version().major >= 4 && !matches!(context, XmlEncodeContext::Plain { .. }) {
        Ok(encode_base64(
            &(instant.unix_timestamp() + EPOCH_SECONDS_FROM_AD).to_le_bytes(),
        ))
    } else {
        Ok(instant.format(&Rfc3339)?)
    }
}

pub trait NodeInstantExt {
    fn get_instant(&self) -> Result<Option<OffsetDateTime>, XmlInstantError>;

    fn add_date_time<E>(
        &mut self,
        context: &XmlEncodeContext<E>,
        instant: Option<OffsetDateTime>,
    ) -> Result<(), XmlInstantError>;
}

#[derive(Debug, thiserror::Error)]
pub enum XmlInstantError {
    #[error(transparent)]
    Base64(#[from] Base64Error),

    #[error(transparent)]
    Parse(#[from] time::error::Parse),

    #[error(transparent)]
    Format(#[from] time::error::Format),

    #[error(transparent)]
    ComponentRange(#[from] time::error::ComponentRange),

    #[error("binary timestamp value must decode to 8 bytes, got {0}")]
    InvalidBinaryTimestampLength(usize),
}

impl PartialEq for XmlInstantError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Base64(left), Self::Base64(right)) => left == right,
            (
                Self::InvalidBinaryTimestampLength(left),
                Self::InvalidBinaryTimestampLength(right),
            ) => left == right,
            (Self::Parse(left), Self::Parse(right)) => left.to_string() == right.to_string(),
            (Self::Format(left), Self::Format(right)) => left.to_string() == right.to_string(),
            (Self::ComponentRange(left), Self::ComponentRange(right)) => {
                left.to_string() == right.to_string()
            }
            _ => false,
        }
    }
}

impl Eq for XmlInstantError {}

impl NodeInstantExt for Node {
    fn get_instant(&self) -> Result<Option<OffsetDateTime>, XmlInstantError> {
        self.get_text().map(parse_instant).transpose()
    }

    fn add_date_time<E>(
        &mut self,
        context: &XmlEncodeContext<E>,
        instant: Option<OffsetDateTime>,
    ) -> Result<(), XmlInstantError> {
        if let Some(instant) = instant {
            self.text(marshal_instant(instant, context)?);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};

    use super::{
        EPOCH_SECONDS_FROM_AD, NodeInstantExt, XmlInstantError, marshal_instant, parse_instant,
    };
    use crate::{
        model::{BinaryData, FormatVersion, XmlEncodeContext},
        xml::{Node, NodeXmlExt},
    };

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct NoEncryption;

    fn plain_context() -> XmlEncodeContext<NoEncryption> {
        XmlEncodeContext::Plain {
            version: FormatVersion::new(4, 1),
            binaries: IndexMap::new(),
            memory_protection_flags: Default::default(),
        }
    }

    fn encrypted_context() -> XmlEncodeContext<NoEncryption> {
        XmlEncodeContext::Encrypted {
            version: FormatVersion::new(4, 1),
            binaries: IndexMap::<Vec<u8>, BinaryData>::new(),
            inner_encryption: NoEncryption,
        }
    }

    #[test]
    fn parses_iso_instant_text() {
        let instant = parse_instant("2020-01-02T03:04:05Z").unwrap();

        assert_eq!(instant.unix_timestamp(), 1_577_934_245);
    }

    #[test]
    fn parses_binary_ad_epoch_timestamp() {
        let instant = parse_instant("APeRdw4AAAA=").unwrap();

        assert_eq!(instant, OffsetDateTime::from_unix_timestamp(0).unwrap());
    }

    #[test]
    fn rejects_binary_timestamp_with_wrong_length() {
        assert_eq!(
            parse_instant("AA=="),
            Err(XmlInstantError::InvalidBinaryTimestampLength(1))
        );
    }

    #[test]
    fn marshals_plain_context_as_iso_instant() {
        let instant = OffsetDateTime::parse("2020-01-02T03:04:05Z", &Rfc3339).unwrap();

        assert_eq!(
            marshal_instant(instant, &plain_context()).unwrap(),
            "2020-01-02T03:04:05Z"
        );
    }

    #[test]
    fn marshals_encrypted_v4_context_as_binary_timestamp() {
        let instant = OffsetDateTime::from_unix_timestamp(0).unwrap();

        assert_eq!(
            marshal_instant(instant, &encrypted_context()).unwrap(),
            "APeRdw4AAAA="
        );
        assert_eq!(
            i64::from_le_bytes(
                crate::io::base64::decode_base64_to_array("APeRdw4AAAA=")
                    .unwrap()
                    .try_into()
                    .unwrap()
            ),
            EPOCH_SECONDS_FROM_AD
        );
    }

    #[test]
    fn reads_and_writes_node_date_time_values() {
        let instant = OffsetDateTime::from_unix_timestamp(0).unwrap();
        let mut node = Node::new("Time");

        node.add_date_time(&encrypted_context(), Some(instant))
            .unwrap();

        assert_eq!(node.get_text(), Some("APeRdw4AAAA="));
        assert_eq!(node.get_instant().unwrap(), Some(instant));
    }

    #[test]
    fn omits_missing_date_time_values() {
        let mut node = Node::new("Time");

        node.add_date_time(&plain_context(), None).unwrap();

        assert_eq!(node.get_text(), None);
    }
}
