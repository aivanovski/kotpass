use std::io::Read;

use crate::{
    error::FormatError,
    model::{DatabaseContent, Meta, XmlDecodeContext, XmlEncodeContext},
    xml::InnerStream,
};

pub trait XmlContentParser {
    fn unmarshal_content_bytes<E, F>(
        &self,
        xml_data: &[u8],
        context_block: F,
    ) -> Result<DatabaseContent, FormatError>
    where
        E: InnerStream,
        F: FnOnce(&Meta) -> XmlDecodeContext<E>;

    fn unmarshal_content_reader<E, F, R>(
        &self,
        source: R,
        context_block: F,
    ) -> Result<DatabaseContent, FormatError>
    where
        E: InnerStream,
        F: FnOnce(&Meta) -> XmlDecodeContext<E>,
        R: Read;

    fn marshal_content<E>(
        &self,
        context: &mut XmlEncodeContext<E>,
        content: &DatabaseContent,
        pretty: bool,
    ) -> Result<String, FormatError>
    where
        E: InnerStream;
}
