use indexmap::IndexSet;

use crate::{
    constants::{MemoryProtectionFlag, defaults},
    error::FormatError,
    model::{Meta, XmlEncodeContext},
    xml::{
        InnerStream, Node, NodeInstantExt, NodeXmlExt, format_xml, marshal_binary_data,
        marshal_custom_data, marshal_custom_icons, unmarshal_binaries, unmarshal_custom_data,
        unmarshal_custom_icons,
    },
};

pub fn unmarshal_meta(node: &Node) -> Result<Meta, FormatError> {
    Ok(Meta {
        generator: text_or_default(node, format_xml::tags::meta::GENERATOR, defaults::GENERATOR),
        header_hash: bytes(node, format_xml::tags::meta::HEADER_HASH)?,
        settings_changed: instant(node, format_xml::tags::meta::SETTINGS_CHANGED)?,
        name: text_or_default(node, format_xml::tags::meta::DATABASE_NAME, ""),
        name_changed: instant(node, format_xml::tags::meta::DATABASE_NAME_CHANGED)?,
        description: text_or_default(node, format_xml::tags::meta::DATABASE_DESCRIPTION, ""),
        description_changed: instant(
            node,
            format_xml::tags::meta::DATABASE_DESCRIPTION_CHANGED,
        )?,
        default_user: text_or_default(node, format_xml::tags::meta::DEFAULT_USER_NAME, ""),
        default_user_changed: instant(node, format_xml::tags::meta::DEFAULT_USER_NAME_CHANGED)?,
        maintenance_history_days: node
            .first(format_xml::tags::meta::MAINTENANCE_HISTORY_DAYS)
            .and_then(NodeXmlExt::get_text)
            .and_then(|text| text.parse::<u32>().ok())
            .unwrap_or(defaults::MAINTENANCE_HISTORY_DAYS),
        color: text(node, format_xml::tags::meta::COLOR),
        master_key_changed: instant(node, format_xml::tags::meta::MASTER_KEY_CHANGED)?,
        master_key_change_rec: int_or_default(
            node,
            format_xml::tags::meta::MASTER_KEY_CHANGE_REC,
            -1,
        )?,
        master_key_change_force: int_or_default(
            node,
            format_xml::tags::meta::MASTER_KEY_CHANGE_FORCE,
            -1,
        )?,
        recycle_bin_enabled: bool_value(node, format_xml::tags::meta::RECYCLE_BIN_ENABLED),
        recycle_bin_uuid: uuid(node, format_xml::tags::meta::RECYCLE_BIN_UUID)?,
        recycle_bin_changed: instant(node, format_xml::tags::meta::RECYCLE_BIN_CHANGED)?,
        entry_templates_group: uuid(node, format_xml::tags::meta::ENTRY_TEMPLATES_GROUP)?,
        entry_templates_group_changed: instant(
            node,
            format_xml::tags::meta::ENTRY_TEMPLATES_GROUP_CHANGED,
        )?,
        history_max_items: int_or_default(
            node,
            format_xml::tags::meta::HISTORY_MAX_ITEMS,
            defaults::HISTORY_MAX_ITEMS,
        )?,
        history_max_size: int_or_default(
            node,
            format_xml::tags::meta::HISTORY_MAX_SIZE,
            defaults::HISTORY_MAX_SIZE,
        )?,
        last_selected_group: uuid(node, format_xml::tags::meta::LAST_SELECTED_GROUP)?,
        last_top_visible_group: uuid(node, format_xml::tags::meta::LAST_TOP_VISIBLE_GROUP)?,
        memory_protection: node
            .first(format_xml::tags::meta::memory_protection::TAG_NAME)
            .map(unmarshal_memory_protection)
            .unwrap_or_default(),
        binaries: node
            .first(format_xml::tags::meta::binaries::TAG_NAME)
            .map(unmarshal_binaries)
            .transpose()?
            .unwrap_or_default(),
        custom_icons: node
            .first(format_xml::tags::meta::custom_icons::TAG_NAME)
            .map(unmarshal_custom_icons)
            .transpose()?
            .unwrap_or_default(),
        custom_data: node
            .first(format_xml::tags::custom_data::TAG_NAME)
            .map(unmarshal_custom_data)
            .transpose()
            .map_err(|error| FormatError::InvalidXml(error.to_string()))?
            .unwrap_or_default(),
    })
}

fn text(node: &Node, name: &str) -> Option<String> {
    node.first(name)
        .and_then(NodeXmlExt::get_text)
        .map(ToOwned::to_owned)
}

fn text_or_default(node: &Node, name: &str, default: &str) -> String {
    text(node, name).unwrap_or_else(|| default.to_owned())
}

fn bool_value(node: &Node, name: &str) -> bool {
    node.first(name)
        .and_then(NodeXmlExt::get_text)
        .is_some_and(|text| text.eq_ignore_ascii_case("true"))
}

fn int_or_default(node: &Node, name: &str, default: i32) -> Result<i32, FormatError> {
    node.first(name)
        .and_then(NodeXmlExt::get_text)
        .map(|text| {
            text.parse::<i32>()
                .map_err(|_| FormatError::InvalidXml(format!("Invalid integer value: {name}.")))
        })
        .transpose()
        .map(|value| value.unwrap_or(default))
}

fn bytes(node: &Node, name: &str) -> Result<Option<Vec<u8>>, FormatError> {
    node.first(name)
        .map(NodeXmlExt::get_bytes)
        .transpose()
        .map_err(|error| FormatError::InvalidXml(error.to_string()))
        .map(Option::flatten)
}

fn uuid(node: &Node, name: &str) -> Result<Option<uuid::Uuid>, FormatError> {
    node.first(name)
        .map(NodeXmlExt::get_uuid)
        .transpose()
        .map_err(|error| FormatError::InvalidXml(error.to_string()))
        .map(Option::flatten)
}

fn instant(node: &Node, name: &str) -> Result<Option<time::OffsetDateTime>, FormatError> {
    node.first(name)
        .map(NodeInstantExt::get_instant)
        .transpose()
        .map_err(|error| FormatError::InvalidXml(error.to_string()))
        .map(Option::flatten)
}

fn unmarshal_memory_protection(node: &Node) -> IndexSet<MemoryProtectionFlag> {
    MemoryProtectionFlag::ALL
        .into_iter()
        .filter(|field| bool_value(node, field.value()))
        .collect()
}

pub fn marshal_meta<E: InnerStream>(
    meta: &Meta,
    context: &mut XmlEncodeContext<E>,
) -> Result<Node, FormatError> {
    let version = context.version();
    let mut node = Node::new(format_xml::tags::meta::TAG_NAME);

    node.element(format_xml::tags::meta::GENERATOR)
        .text(&meta.generator);
    if version.major < 4 {
        if let Some(header_hash) = &meta.header_hash {
            node.element(format_xml::tags::meta::HEADER_HASH)
                .add_bytes(header_hash);
        }
    }
    if version.major >= 4 && meta.settings_changed.is_some() {
        node.element(format_xml::tags::meta::SETTINGS_CHANGED)
            .add_date_time(context, meta.settings_changed)
            .map_err(|error| FormatError::InvalidXml(error.to_string()))?;
    }
    node.element(format_xml::tags::meta::DATABASE_NAME)
        .text(&meta.name);
    node.element(format_xml::tags::meta::DATABASE_NAME_CHANGED)
        .add_date_time(context, meta.name_changed)
        .map_err(|error| FormatError::InvalidXml(error.to_string()))?;
    node.element(format_xml::tags::meta::DATABASE_DESCRIPTION)
        .text(&meta.description);
    node.element(format_xml::tags::meta::DATABASE_DESCRIPTION_CHANGED)
        .add_date_time(context, meta.description_changed)
        .map_err(|error| FormatError::InvalidXml(error.to_string()))?;
    node.element(format_xml::tags::meta::DEFAULT_USER_NAME)
        .text(&meta.default_user);
    node.element(format_xml::tags::meta::DEFAULT_USER_NAME_CHANGED)
        .add_date_time(context, meta.default_user_changed)
        .map_err(|error| FormatError::InvalidXml(error.to_string()))?;
    node.element(format_xml::tags::meta::MAINTENANCE_HISTORY_DAYS)
        .text(meta.maintenance_history_days.to_string());
    let color = node.element(format_xml::tags::meta::COLOR);
    if let Some(value) = &meta.color {
        color.text(value);
    }
    node.element(format_xml::tags::meta::MASTER_KEY_CHANGED)
        .add_date_time(context, meta.master_key_changed)
        .map_err(|error| FormatError::InvalidXml(error.to_string()))?;
    node.element(format_xml::tags::meta::MASTER_KEY_CHANGE_REC)
        .text(meta.master_key_change_rec.to_string());
    node.element(format_xml::tags::meta::MASTER_KEY_CHANGE_FORCE)
        .text(meta.master_key_change_force.to_string());
    node.element(format_xml::tags::meta::RECYCLE_BIN_ENABLED)
        .add_boolean(meta.recycle_bin_enabled);
    let recycle_bin_uuid = node.element(format_xml::tags::meta::RECYCLE_BIN_UUID);
    if let Some(value) = meta.recycle_bin_uuid {
        recycle_bin_uuid.add_uuid(value);
    }
    node.element(format_xml::tags::meta::RECYCLE_BIN_CHANGED)
        .add_date_time(context, meta.recycle_bin_changed)
        .map_err(|error| FormatError::InvalidXml(error.to_string()))?;
    let entry_templates_group = node.element(format_xml::tags::meta::ENTRY_TEMPLATES_GROUP);
    if let Some(value) = meta.entry_templates_group {
        entry_templates_group.add_uuid(value);
    }
    node.element(format_xml::tags::meta::ENTRY_TEMPLATES_GROUP_CHANGED)
        .add_date_time(context, meta.entry_templates_group_changed)
        .map_err(|error| FormatError::InvalidXml(error.to_string()))?;
    node.element(format_xml::tags::meta::HISTORY_MAX_ITEMS)
        .text(meta.history_max_items.to_string());
    node.element(format_xml::tags::meta::HISTORY_MAX_SIZE)
        .text(meta.history_max_size.to_string());
    let last_selected_group = node.element(format_xml::tags::meta::LAST_SELECTED_GROUP);
    if let Some(value) = meta.last_selected_group {
        last_selected_group.add_uuid(value);
    }
    let last_top_visible_group = node.element(format_xml::tags::meta::LAST_TOP_VISIBLE_GROUP);
    if let Some(value) = meta.last_top_visible_group {
        last_top_visible_group.add_uuid(value);
    }

    node.add_element(marshal_memory_protection(&meta.memory_protection));
    node.add_element(
        marshal_custom_icons(context, &meta.custom_icons)
            .map_err(|error| FormatError::InvalidXml(error.to_string()))?,
    );
    node.add_element(
        marshal_custom_data(context, &meta.custom_data)
            .map_err(|error| FormatError::InvalidXml(error.to_string()))?,
    );

    if version.major < 4 || matches!(context, XmlEncodeContext::Plain { .. }) {
        let binaries = node.element(format_xml::tags::meta::binaries::TAG_NAME);
        for (id, binary) in context.binaries().values().enumerate() {
            binaries.add_element(marshal_binary_data(binary, id as i32));
        }
    }

    Ok(node)
}

fn marshal_memory_protection(memory_protection: &IndexSet<MemoryProtectionFlag>) -> Node {
    let mut node = Node::new(format_xml::tags::meta::memory_protection::TAG_NAME);
    for field in MemoryProtectionFlag::ALL {
        node.element(field.value())
            .add_boolean(memory_protection.contains(&field));
    }
    node
}
