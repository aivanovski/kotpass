use indexmap::IndexMap;

use crate::{
    model::{CustomDataValue, XmlEncodeContext},
    xml::{Node, NodeInstantExt, NodeXmlExt, XmlInstantError, format_xml},
};

pub fn unmarshal_custom_data(
    node: &Node,
) -> Result<IndexMap<String, CustomDataValue>, XmlInstantError> {
    let mut custom_data = IndexMap::new();
    for item in node
        .child_nodes()
        .into_iter()
        .filter(|node| node.node_name == format_xml::tags::custom_data::ITEM)
    {
        if let Some((key, value)) = unmarshal_custom_data_item(item)? {
            custom_data.insert(key, value);
        }
    }
    Ok(custom_data)
}

fn unmarshal_custom_data_item(
    node: &Node,
) -> Result<Option<(String, CustomDataValue)>, XmlInstantError> {
    let Some(key) = node
        .first(format_xml::tags::custom_data::ITEM_KEY)
        .and_then(NodeXmlExt::get_text)
    else {
        return Ok(None);
    };
    let Some(value) = node
        .first(format_xml::tags::custom_data::ITEM_VALUE)
        .and_then(NodeXmlExt::get_text)
    else {
        return Ok(None);
    };
    let last_modified = node
        .first(format_xml::tags::time_data::LAST_MODIFICATION_TIME)
        .map(NodeInstantExt::get_instant)
        .transpose()?
        .flatten();

    Ok(Some((
        key.to_owned(),
        CustomDataValue {
            value: value.to_owned(),
            last_modified,
        },
    )))
}

pub fn marshal_custom_data<E>(
    context: &XmlEncodeContext<E>,
    custom_data: &IndexMap<String, CustomDataValue>,
) -> Result<Node, XmlInstantError> {
    let mut node = Node::new(format_xml::tags::custom_data::TAG_NAME);

    for (key, item) in custom_data {
        let item_node = node.element(format_xml::tags::custom_data::ITEM);
        item_node
            .element(format_xml::tags::custom_data::ITEM_KEY)
            .text(key);
        item_node
            .element(format_xml::tags::custom_data::ITEM_VALUE)
            .text(&item.value);

        if context.version().is_at_least(4, 1) {
            item_node
                .element(format_xml::tags::time_data::LAST_MODIFICATION_TIME)
                .add_date_time(context, item.last_modified)?;
        }
    }

    Ok(node)
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};

    use super::{marshal_custom_data, unmarshal_custom_data};
    use crate::{
        model::{BinaryData, CustomDataValue, FormatVersion, XmlEncodeContext},
        xml::{Node, NodeXmlExt, format_xml},
    };

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct NoEncryption;

    fn context(version: FormatVersion) -> XmlEncodeContext<NoEncryption> {
        XmlEncodeContext::Plain {
            version,
            binaries: IndexMap::<Vec<u8>, BinaryData>::new(),
            memory_protection_flags: Default::default(),
        }
    }

    #[test]
    fn unmarshals_complete_items_and_skips_incomplete_items() {
        let mut root = Node::new(format_xml::tags::custom_data::TAG_NAME);
        let item = root.element(format_xml::tags::custom_data::ITEM);
        item.element(format_xml::tags::custom_data::ITEM_KEY)
            .text("key");
        item.element(format_xml::tags::custom_data::ITEM_VALUE)
            .text("value");
        item.element(format_xml::tags::time_data::LAST_MODIFICATION_TIME)
            .text("2020-01-02T03:04:05Z");
        root.element(format_xml::tags::custom_data::ITEM)
            .element(format_xml::tags::custom_data::ITEM_KEY)
            .text("missing value");

        let data = unmarshal_custom_data(&root).unwrap();

        assert_eq!(data.len(), 1);
        assert_eq!(data["key"].value, "value");
        assert_eq!(
            data["key"].last_modified.unwrap().unix_timestamp(),
            1_577_934_245
        );
    }

    #[test]
    fn marshals_v41_custom_data_with_last_modified_time() {
        let instant = OffsetDateTime::parse("2020-01-02T03:04:05Z", &Rfc3339).unwrap();
        let mut data = IndexMap::new();
        data.insert(
            "key".to_owned(),
            CustomDataValue::with_last_modified("value", instant),
        );

        let node = marshal_custom_data(&context(FormatVersion::new(4, 1)), &data).unwrap();
        let item = node.first(format_xml::tags::custom_data::ITEM).unwrap();

        assert_eq!(
            item.first(format_xml::tags::custom_data::ITEM_KEY)
                .and_then(NodeXmlExt::get_text),
            Some("key")
        );
        assert_eq!(
            item.first(format_xml::tags::time_data::LAST_MODIFICATION_TIME)
                .and_then(NodeXmlExt::get_text),
            Some("2020-01-02T03:04:05Z")
        );
    }

    #[test]
    fn omits_last_modified_time_before_v41() {
        let mut data = IndexMap::new();
        data.insert("key".to_owned(), CustomDataValue::new("value"));

        let node = marshal_custom_data(&context(FormatVersion::new(4, 0)), &data).unwrap();
        let item = node.first(format_xml::tags::custom_data::ITEM).unwrap();

        assert!(
            item.first(format_xml::tags::time_data::LAST_MODIFICATION_TIME)
                .is_none()
        );
    }
}
