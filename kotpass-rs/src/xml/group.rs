use crate::{
    constants::{PredefinedIcon, consts},
    error::FormatError,
    model::{Group, XmlDecodeContext, XmlEncodeContext},
    xml::{
        InnerStream, Node, NodeXmlExt, format_xml, marshal_custom_data, marshal_entry,
        marshal_time_data, unmarshal_custom_data, unmarshal_entry, unmarshal_time_data,
    },
};

pub fn unmarshal_group<E: InnerStream>(
    context: &mut XmlDecodeContext<E>,
    node: &Node,
) -> Result<Group, FormatError> {
    let uuid = node
        .first(format_xml::tags::UUID)
        .map(NodeXmlExt::get_uuid)
        .transpose()
        .map_err(|error| FormatError::InvalidXml(error.to_string()))?
        .flatten()
        .ok_or_else(|| FormatError::InvalidXml("Invalid entry without Uuid.".to_owned()))?;
    let mut group = Group::new(uuid, "");

    for child in node.child_nodes() {
        match child.node_name.as_str() {
            format_xml::tags::group::NAME => {
                group.name = child.get_text().unwrap_or("").to_owned();
            }
            format_xml::tags::group::NOTES => {
                group.notes = child.get_text().unwrap_or("").to_owned();
            }
            format_xml::tags::group::ICON_ID => {
                group.icon = child
                    .get_text()
                    .and_then(|text| text.parse::<usize>().ok())
                    .and_then(PredefinedIcon::from_ordinal)
                    .unwrap_or(PredefinedIcon::Folder);
            }
            format_xml::tags::group::CUSTOM_ICON_ID => {
                group.custom_icon_uuid = child
                    .get_uuid()
                    .map_err(|error| FormatError::InvalidXml(error.to_string()))?;
            }
            format_xml::tags::time_data::TAG_NAME => {
                group.times = Some(unmarshal_time_data(child)?);
            }
            format_xml::tags::group::IS_EXPANDED => {
                group.expanded = child
                    .get_text()
                    .is_some_and(|text| text.eq_ignore_ascii_case("true"));
            }
            format_xml::tags::group::DEFAULT_AUTO_TYPE_SEQUENCE => {
                group.default_auto_type_sequence = child.get_text().map(ToOwned::to_owned);
            }
            format_xml::tags::group::ENABLE_AUTO_TYPE => {
                group.enable_auto_type = child.get_group_override();
            }
            format_xml::tags::group::ENABLE_SEARCHING => {
                group.enable_searching = child.get_group_override();
            }
            format_xml::tags::group::LAST_TOP_VISIBLE_ENTRY => {
                group.last_top_visible_entry = child
                    .get_uuid()
                    .map_err(|error| FormatError::InvalidXml(error.to_string()))?;
            }
            format_xml::tags::group::PREVIOUS_PARENT_GROUP => {
                group.previous_parent_group = child
                    .get_uuid()
                    .map_err(|error| FormatError::InvalidXml(error.to_string()))?;
            }
            format_xml::tags::group::TAGS => {
                if let Some(tags) = child.get_text() {
                    group
                        .tags
                        .extend(consts::split_tags(tags).into_iter().map(ToOwned::to_owned));
                }
            }
            format_xml::tags::group::TAG_NAME => {
                group.groups.push(unmarshal_group(context, child)?);
            }
            format_xml::tags::entry::TAG_NAME => {
                group.entries.push(unmarshal_entry(context, child)?);
            }
            format_xml::tags::custom_data::TAG_NAME => {
                group.custom_data = unmarshal_custom_data(child)
                    .map_err(|error| FormatError::InvalidXml(error.to_string()))?;
            }
            _ => {}
        }
    }

    Ok(group)
}

pub fn marshal_group<E: InnerStream>(
    group: &Group,
    context: &mut XmlEncodeContext<E>,
) -> Result<Node, FormatError> {
    let mut node = Node::new(format_xml::tags::group::TAG_NAME);
    node.element(format_xml::tags::UUID).add_uuid(group.uuid);
    node.element(format_xml::tags::group::NAME).text(&group.name);
    node.element(format_xml::tags::group::NOTES)
        .text(&group.notes);
    node.element(format_xml::tags::group::ICON_ID)
        .text(group.icon.ordinal().to_string());
    if let Some(custom_icon_uuid) = group.custom_icon_uuid {
        node.element(format_xml::tags::group::CUSTOM_ICON_ID)
            .add_uuid(custom_icon_uuid);
    }
    if let Some(times) = &group.times {
        node.add_element(
            marshal_time_data(times, context)
                .map_err(|error| FormatError::InvalidXml(error.to_string()))?,
        );
    }
    node.element(format_xml::tags::group::IS_EXPANDED)
        .add_boolean(group.expanded);
    node.element(format_xml::tags::group::DEFAULT_AUTO_TYPE_SEQUENCE)
        .text(group.default_auto_type_sequence.as_deref().unwrap_or(""));
    node.element(format_xml::tags::group::ENABLE_AUTO_TYPE)
        .add_group_override(group.enable_auto_type);
    node.element(format_xml::tags::group::ENABLE_SEARCHING)
        .add_group_override(group.enable_searching);
    if let Some(last_top_visible_entry) = group.last_top_visible_entry {
        node.element(format_xml::tags::group::LAST_TOP_VISIBLE_ENTRY)
            .add_uuid(last_top_visible_entry);
    }
    if context.version().is_at_least(4, 1) {
        if let Some(previous_parent_group) = group.previous_parent_group {
            node.element(format_xml::tags::group::PREVIOUS_PARENT_GROUP)
                .add_uuid(previous_parent_group);
        }
    }
    if context.version().is_at_least(4, 1) {
        node.element(format_xml::tags::group::TAGS)
            .text(group.tags.join(consts::TAGS_SEPARATOR));
    }
    if !group.custom_data.is_empty() {
        node.add_element(
            marshal_custom_data(context, &group.custom_data)
                .map_err(|error| FormatError::InvalidXml(error.to_string()))?,
        );
    }
    for child_group in &group.groups {
        node.add_element(marshal_group(child_group, context)?);
    }
    for entry in &group.entries {
        node.add_element(marshal_entry(entry, context)?);
    }

    Ok(node)
}
