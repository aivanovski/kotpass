use crate::{
    error::FormatError,
    model::{TimeData, XmlEncodeContext},
    xml::{Node, NodeInstantExt, NodeXmlExt, XmlInstantError, format_xml},
};

pub fn unmarshal_time_data(node: &Node) -> Result<TimeData, FormatError> {
    Ok(TimeData {
        creation_time: get_optional_instant(node, format_xml::tags::time_data::CREATION_TIME)?,
        last_access_time: get_optional_instant(
            node,
            format_xml::tags::time_data::LAST_ACCESS_TIME,
        )?,
        last_modification_time: get_optional_instant(
            node,
            format_xml::tags::time_data::LAST_MODIFICATION_TIME,
        )?,
        location_changed: get_optional_instant(
            node,
            format_xml::tags::time_data::LOCATION_CHANGED,
        )?,
        expiry_time: get_optional_instant(node, format_xml::tags::time_data::EXPIRY_TIME)?,
        expires: node
            .first(format_xml::tags::time_data::EXPIRES)
            .and_then(NodeXmlExt::get_text)
            .is_some_and(|text| text.eq_ignore_ascii_case("true")),
        usage_count: node
            .first(format_xml::tags::time_data::USAGE_COUNT)
            .and_then(NodeXmlExt::get_text)
            .map(|text| {
                text.parse::<i32>()
                    .map_err(|_| FormatError::InvalidXml("Invalid usage count.".to_owned()))
            })
            .transpose()?
            .unwrap_or(0),
    })
}

fn get_optional_instant(
    node: &Node,
    name: &str,
) -> Result<Option<time::OffsetDateTime>, FormatError> {
    node.first(name)
        .map(NodeInstantExt::get_instant)
        .transpose()
        .map_err(|error| FormatError::InvalidXml(error.to_string()))
        .map(Option::flatten)
}

pub fn marshal_time_data<E>(
    time_data: &TimeData,
    context: &XmlEncodeContext<E>,
) -> Result<Node, XmlInstantError> {
    let mut node = Node::new(format_xml::tags::time_data::TAG_NAME);
    node.element(format_xml::tags::time_data::CREATION_TIME)
        .add_date_time(context, time_data.creation_time)?;
    node.element(format_xml::tags::time_data::LAST_ACCESS_TIME)
        .add_date_time(context, time_data.last_access_time)?;
    node.element(format_xml::tags::time_data::LAST_MODIFICATION_TIME)
        .add_date_time(context, time_data.last_modification_time)?;
    node.element(format_xml::tags::time_data::LOCATION_CHANGED)
        .add_date_time(context, time_data.location_changed)?;
    node.element(format_xml::tags::time_data::EXPIRY_TIME)
        .add_date_time(context, time_data.expiry_time)?;
    node.element(format_xml::tags::time_data::EXPIRES)
        .add_boolean(time_data.expires);
    node.element(format_xml::tags::time_data::USAGE_COUNT)
        .text(time_data.usage_count.to_string());
    Ok(node)
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};

    use super::{marshal_time_data, unmarshal_time_data};
    use crate::{
        model::{BinaryData, FormatVersion, TimeData, XmlEncodeContext},
        xml::{Node, NodeXmlExt, format_xml},
    };

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct NoEncryption;

    fn context() -> XmlEncodeContext<NoEncryption> {
        XmlEncodeContext::Plain {
            version: FormatVersion::new(4, 1),
            binaries: IndexMap::<Vec<u8>, BinaryData>::new(),
            memory_protection_flags: Default::default(),
        }
    }

    #[test]
    fn unmarshals_time_data_with_defaults() {
        let mut node = Node::new(format_xml::tags::time_data::TAG_NAME);
        node.element(format_xml::tags::time_data::CREATION_TIME)
            .text("2020-01-02T03:04:05Z");
        node.element(format_xml::tags::time_data::EXPIRES)
            .text("True");
        node.element(format_xml::tags::time_data::USAGE_COUNT)
            .text("7");

        let data = unmarshal_time_data(&node).unwrap();

        assert_eq!(data.creation_time.unwrap().unix_timestamp(), 1_577_934_245);
        assert_eq!(data.last_access_time, None);
        assert!(data.expires);
        assert_eq!(data.usage_count, 7);
    }

    #[test]
    fn defaults_missing_bool_and_usage_count() {
        let data = unmarshal_time_data(&Node::new(format_xml::tags::time_data::TAG_NAME)).unwrap();

        assert!(!data.expires);
        assert_eq!(data.usage_count, 0);
    }

    #[test]
    fn marshals_time_data_like_kotlin_mapper() {
        let instant = OffsetDateTime::parse("2020-01-02T03:04:05Z", &Rfc3339).unwrap();
        let data = TimeData {
            creation_time: Some(instant),
            last_access_time: None,
            last_modification_time: Some(instant),
            location_changed: None,
            expiry_time: None,
            expires: true,
            usage_count: 7,
        };

        let node = marshal_time_data(&data, &context()).unwrap();

        assert_eq!(node.node_name, format_xml::tags::time_data::TAG_NAME);
        assert_eq!(
            node.first(format_xml::tags::time_data::CREATION_TIME)
                .and_then(NodeXmlExt::get_text),
            Some("2020-01-02T03:04:05Z")
        );
        assert_eq!(
            node.first(format_xml::tags::time_data::LAST_ACCESS_TIME)
                .and_then(NodeXmlExt::get_text),
            None
        );
        assert_eq!(
            node.first(format_xml::tags::time_data::EXPIRES)
                .and_then(NodeXmlExt::get_text),
            Some("True")
        );
        assert_eq!(
            node.first(format_xml::tags::time_data::USAGE_COUNT)
                .and_then(NodeXmlExt::get_text),
            Some("7")
        );
    }
}
