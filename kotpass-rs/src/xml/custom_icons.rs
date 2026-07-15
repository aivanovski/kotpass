use indexmap::IndexMap;
use uuid::Uuid;

use crate::{
    error::FormatError,
    model::{CustomIcon, XmlEncodeContext},
    xml::{Node, NodeInstantExt, NodeXmlExt, XmlInstantError, format_xml},
};

pub fn unmarshal_custom_icons(node: &Node) -> Result<IndexMap<Uuid, CustomIcon>, FormatError> {
    let mut custom_icons = IndexMap::new();
    for item in node
        .child_nodes()
        .into_iter()
        .filter(|node| node.node_name == format_xml::tags::meta::custom_icons::ITEM)
    {
        if let Some((id, icon)) = unmarshal_custom_icon(item)? {
            custom_icons.insert(id, icon);
        }
    }
    Ok(custom_icons)
}

fn unmarshal_custom_icon(node: &Node) -> Result<Option<(Uuid, CustomIcon)>, FormatError> {
    let Some(id_node) = node.first(format_xml::tags::meta::custom_icons::ITEM_UUID) else {
        return Ok(None);
    };
    let Some(id) = id_node
        .get_uuid()
        .map_err(|error| FormatError::InvalidXml(error.to_string()))?
    else {
        return Ok(None);
    };
    let Some(data_node) = node.first(format_xml::tags::meta::custom_icons::ITEM_DATA) else {
        return Ok(None);
    };
    let Some(data) = data_node
        .get_bytes()
        .map_err(|error| FormatError::InvalidXml(error.to_string()))?
    else {
        return Ok(None);
    };
    let name = node
        .first(format_xml::tags::meta::custom_icons::ITEM_NAME)
        .and_then(NodeXmlExt::get_text)
        .map(ToOwned::to_owned);
    let last_modified = node
        .first(format_xml::tags::time_data::LAST_MODIFICATION_TIME)
        .map(NodeInstantExt::get_instant)
        .transpose()
        .map_err(|error| FormatError::InvalidXml(error.to_string()))?
        .flatten();

    Ok(Some((id, CustomIcon::new(data, name, last_modified))))
}

pub fn marshal_custom_icons<E>(
    context: &XmlEncodeContext<E>,
    custom_icons: &IndexMap<Uuid, CustomIcon>,
) -> Result<Node, XmlInstantError> {
    let mut node = Node::new(format_xml::tags::meta::custom_icons::TAG_NAME);

    for (id, icon) in custom_icons {
        let item = node.element(format_xml::tags::meta::custom_icons::ITEM);
        item.element(format_xml::tags::meta::custom_icons::ITEM_UUID)
            .add_uuid(*id);
        item.element(format_xml::tags::meta::custom_icons::ITEM_DATA)
            .add_bytes(&icon.data);

        if context.version().is_at_least(4, 1) {
            let name = item.element(format_xml::tags::meta::custom_icons::ITEM_NAME);
            if let Some(value) = &icon.name {
                name.text(value);
            }
            item.element(format_xml::tags::time_data::LAST_MODIFICATION_TIME)
                .add_date_time(context, icon.last_modified)?;
        }
    }

    Ok(node)
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};
    use uuid::Uuid;

    use super::{marshal_custom_icons, unmarshal_custom_icons};
    use crate::{
        model::{BinaryData, CustomIcon, FormatVersion, XmlEncodeContext},
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
    fn unmarshals_custom_icons_and_skips_incomplete_items() {
        let id = Uuid::parse_str("12345678-9abc-def0-1234-56789abcdef0").unwrap();
        let mut root = Node::new(format_xml::tags::meta::custom_icons::TAG_NAME);
        let item = root.element(format_xml::tags::meta::custom_icons::ITEM);
        item.element(format_xml::tags::meta::custom_icons::ITEM_UUID)
            .add_uuid(id);
        item.element(format_xml::tags::meta::custom_icons::ITEM_DATA)
            .add_bytes(b"png");
        item.element(format_xml::tags::meta::custom_icons::ITEM_NAME)
            .text("icon");
        item.element(format_xml::tags::time_data::LAST_MODIFICATION_TIME)
            .text("2020-01-02T03:04:05Z");
        root.element(format_xml::tags::meta::custom_icons::ITEM)
            .element(format_xml::tags::meta::custom_icons::ITEM_UUID)
            .add_uuid(Uuid::from_u128(0));

        let icons = unmarshal_custom_icons(&root).unwrap();

        assert_eq!(icons.len(), 1);
        assert_eq!(icons[&id].data, b"png");
        assert_eq!(icons[&id].name.as_deref(), Some("icon"));
        assert_eq!(
            icons[&id].last_modified.unwrap().unix_timestamp(),
            1_577_934_245
        );
    }

    #[test]
    fn marshals_v41_custom_icon_metadata() {
        let id = Uuid::parse_str("12345678-9abc-def0-1234-56789abcdef0").unwrap();
        let instant = OffsetDateTime::parse("2020-01-02T03:04:05Z", &Rfc3339).unwrap();
        let mut icons = IndexMap::new();
        icons.insert(
            id,
            CustomIcon::new(b"png".to_vec(), Some("icon".to_owned()), Some(instant)),
        );

        let node = marshal_custom_icons(&context(FormatVersion::new(4, 1)), &icons).unwrap();
        let item = node.first(format_xml::tags::meta::custom_icons::ITEM).unwrap();

        assert_eq!(
            item.first(format_xml::tags::meta::custom_icons::ITEM_UUID)
                .and_then(NodeXmlExt::get_text),
            Some("EjRWeJq83vASNFZ4mrze8A==")
        );
        assert_eq!(
            item.first(format_xml::tags::meta::custom_icons::ITEM_DATA)
                .unwrap()
                .get_bytes()
                .unwrap(),
            Some(b"png".to_vec())
        );
        assert_eq!(
            item.first(format_xml::tags::meta::custom_icons::ITEM_NAME)
                .and_then(NodeXmlExt::get_text),
            Some("icon")
        );
    }

    #[test]
    fn omits_v41_custom_icon_metadata_before_v41() {
        let id = Uuid::from_u128(1);
        let mut icons = IndexMap::new();
        icons.insert(id, CustomIcon::new(b"png".to_vec(), Some("icon".to_owned()), None));

        let node = marshal_custom_icons(&context(FormatVersion::new(4, 0)), &icons).unwrap();
        let item = node.first(format_xml::tags::meta::custom_icons::ITEM).unwrap();

        assert!(
            item.first(format_xml::tags::meta::custom_icons::ITEM_NAME)
                .is_none()
        );
    }
}
