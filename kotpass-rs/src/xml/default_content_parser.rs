use std::io::Read;

use crate::{
    error::FormatError,
    model::{DatabaseContent, Meta, XmlDecodeContext, XmlEncodeContext},
    xml::{
        InnerStream, NodeXmlExt, PrintOptions, XmlContentParser, XmlVersion, format_xml,
        marshal_deleted_object, marshal_group, marshal_meta, parse_str, unmarshal_deleted_object,
        unmarshal_group, unmarshal_meta, xml,
    },
};

pub struct DefaultXmlContentParser;

pub const DEFAULT_XML_CONTENT_PARSER: DefaultXmlContentParser = DefaultXmlContentParser;

impl XmlContentParser for DefaultXmlContentParser {
    fn unmarshal_content_bytes<E, F>(
        &self,
        xml_data: &[u8],
        context_block: F,
    ) -> Result<DatabaseContent, FormatError>
    where
        E: InnerStream,
        F: FnOnce(&Meta) -> XmlDecodeContext<E>,
    {
        let text = std::str::from_utf8(xml_data)
            .map_err(|error| FormatError::InvalidXml(error.to_string()))?;
        self.unmarshal_content_str(text, context_block)
    }

    fn unmarshal_content_reader<E, F, R>(
        &self,
        mut source: R,
        context_block: F,
    ) -> Result<DatabaseContent, FormatError>
    where
        E: InnerStream,
        F: FnOnce(&Meta) -> XmlDecodeContext<E>,
        R: Read,
    {
        let mut text = String::new();
        source
            .read_to_string(&mut text)
            .map_err(|error| FormatError::InvalidXml(error.to_string()))?;
        self.unmarshal_content_str(&text, context_block)
    }

    fn marshal_content<E>(
        &self,
        context: &mut XmlEncodeContext<E>,
        content: &DatabaseContent,
        pretty: bool,
    ) -> Result<String, FormatError>
    where
        E: InnerStream,
    {
        let mut document = xml(
            format_xml::tags::DOCUMENT,
            Some("utf-8"),
            Some(XmlVersion::V10),
            None,
        );
        document.add_element(marshal_meta(&content.meta, context)?);

        let root = document.element(format_xml::tags::ROOT);
        root.add_element(marshal_group(&content.group, context)?);
        let deleted_objects = root.element(format_xml::tags::deleted_objects::TAG_NAME);
        for deleted_object in &content.deleted_objects {
            deleted_objects.add_element(
                marshal_deleted_object(deleted_object, context)
                    .map_err(|error| FormatError::InvalidXml(error.to_string()))?,
            );
        }

        Ok(document.to_string_with_options(PrintOptions {
            pretty,
            single_line_text_elements: true,
            ..PrintOptions::default()
        }))
    }
}

impl DefaultXmlContentParser {
    pub fn unmarshal_content_str<E, F>(
        &self,
        xml_data: &str,
        context_block: F,
    ) -> Result<DatabaseContent, FormatError>
    where
        E: InnerStream,
        F: FnOnce(&Meta) -> XmlDecodeContext<E>,
    {
        let document = parse_str(xml_data)
            .map_err(|error| FormatError::InvalidXml(error.to_string()))?;
        let root = document
            .first(format_xml::tags::ROOT)
            .ok_or_else(|| FormatError::InvalidXml("No root found.".to_owned()))?;
        let meta = document
            .first(format_xml::tags::meta::TAG_NAME)
            .map(unmarshal_meta)
            .transpose()?
            .ok_or_else(|| FormatError::InvalidXml("No metadata found.".to_owned()))?;
        let mut context = context_block(&meta);
        let group = root
            .first(format_xml::tags::group::TAG_NAME)
            .map(|node| unmarshal_group(&mut context, node))
            .transpose()?
            .ok_or_else(|| FormatError::InvalidXml("No root group.".to_owned()))?;
        let deleted_objects = root
            .first(format_xml::tags::deleted_objects::TAG_NAME)
            .map(|node| {
                node.child_nodes()
                    .into_iter()
                    .filter(|node| node.node_name == format_xml::tags::deleted_objects::OBJECT)
                    .map(unmarshal_deleted_object)
                    .filter_map(|result| result.transpose())
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?
            .unwrap_or_default();

        Ok(DatabaseContent::new(meta, group, deleted_objects))
    }
}
