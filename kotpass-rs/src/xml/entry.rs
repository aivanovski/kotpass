use crate::{
    constants::{PredefinedIcon, consts},
    crypto::{EncryptedValue, EncryptionSaltGenerator},
    error::{CryptoError, FormatError},
    model::{Entry, EntryFields, EntryValue, XmlDecodeContext, XmlEncodeContext},
    xml::{
        Node, NodeXmlExt, format_xml, marshal_auto_type_data, marshal_binary_reference,
        marshal_custom_data, marshal_time_data, unmarshal_auto_type_data, unmarshal_binary_reference,
        unmarshal_custom_data, unmarshal_time_data,
    },
};

pub trait InnerStream {
    fn get_salt(&mut self, length: usize) -> Result<Vec<u8>, CryptoError>;
    fn process_bytes(&mut self, input: &[u8]) -> Result<Vec<u8>, CryptoError>;
}

impl InnerStream for EncryptionSaltGenerator {
    fn get_salt(&mut self, length: usize) -> Result<Vec<u8>, CryptoError> {
        EncryptionSaltGenerator::get_salt(self, length)
    }

    fn process_bytes(&mut self, input: &[u8]) -> Result<Vec<u8>, CryptoError> {
        EncryptionSaltGenerator::process_bytes(self, input)
    }
}

pub fn unmarshal_entry<E: InnerStream>(
    context: &mut XmlDecodeContext<E>,
    node: &Node,
) -> Result<Entry, FormatError> {
    let uuid = node
        .first(format_xml::tags::UUID)
        .map(NodeXmlExt::get_uuid)
        .transpose()
        .map_err(|error| FormatError::InvalidXml(error.to_string()))?
        .flatten()
        .ok_or_else(|| FormatError::InvalidXml("Invalid entry without Uuid.".to_owned()))?;
    let mut entry = Entry::new(uuid);
    let mut untitled_fields = Vec::new();

    for child in node.child_nodes() {
        match child.node_name.as_str() {
            format_xml::tags::entry::ICON_ID => {
                entry.icon = child
                    .get_text()
                    .and_then(|text| text.parse::<usize>().ok())
                    .and_then(PredefinedIcon::from_ordinal)
                    .unwrap_or(PredefinedIcon::Key);
            }
            format_xml::tags::entry::CUSTOM_ICON_ID => {
                entry.custom_icon_uuid = child
                    .get_uuid()
                    .map_err(|error| FormatError::InvalidXml(error.to_string()))?;
            }
            format_xml::tags::entry::FOREGROUND_COLOR => {
                entry.foreground_color = child.get_text().map(ToOwned::to_owned);
            }
            format_xml::tags::entry::BACKGROUND_COLOR => {
                entry.background_color = child.get_text().map(ToOwned::to_owned);
            }
            format_xml::tags::entry::OVERRIDE_URL => {
                entry.override_url = child.get_text().unwrap_or("").to_owned();
            }
            format_xml::tags::time_data::TAG_NAME => {
                entry.times = Some(unmarshal_time_data(child)?);
            }
            format_xml::tags::entry::auto_type::TAG_NAME => {
                entry.auto_type = Some(unmarshal_auto_type_data(child));
            }
            format_xml::tags::entry::fields::TAG_NAME => {
                let (name, value) = unmarshal_field(context, child)?;
                if let Some(name) = name {
                    entry.fields.insert(name, value);
                } else {
                    untitled_fields.push(value);
                }
            }
            format_xml::tags::entry::TAGS => {
                if let Some(tags) = child.get_text() {
                    entry
                        .tags
                        .extend(consts::split_tags(tags).into_iter().map(ToOwned::to_owned));
                }
            }
            format_xml::tags::entry::binary_references::TAG_NAME => {
                if let Some(reference) = unmarshal_binary_reference(context, child)? {
                    entry.binaries.push(reference);
                }
            }
            format_xml::tags::entry::HISTORY => {
                entry.history = unmarshal_entries(context, child)?;
            }
            format_xml::tags::custom_data::TAG_NAME => {
                entry.custom_data = unmarshal_custom_data(child)
                    .map_err(|error| FormatError::InvalidXml(error.to_string()))?;
            }
            format_xml::tags::entry::PREVIOUS_PARENT_GROUP => {
                entry.previous_parent_group = child
                    .get_uuid()
                    .map_err(|error| FormatError::InvalidXml(error.to_string()))?;
            }
            format_xml::tags::entry::QUALITY_CHECK => {
                entry.quality_check = child
                    .get_text()
                    .map(|text| text.eq_ignore_ascii_case("true"))
                    .unwrap_or(true);
            }
            _ => {}
        }
    }

    recover_untitled_fields(&mut entry, &context.untitled_label, untitled_fields);
    Ok(entry)
}

pub fn unmarshal_entries<E: InnerStream>(
    context: &mut XmlDecodeContext<E>,
    node: &Node,
) -> Result<Vec<Entry>, FormatError> {
    node.child_nodes()
        .into_iter()
        .filter(|node| node.node_name == format_xml::tags::entry::TAG_NAME)
        .map(|node| unmarshal_entry(context, node))
        .collect()
}

fn recover_untitled_fields(
    entry: &mut Entry,
    untitled_label: &str,
    untitled_fields: Vec<EntryValue>,
) {
    for value in untitled_fields {
        let mut n = 1u32;
        let mut name = untitled_label.to_owned();
        while entry.fields.get(&name).is_some() {
            name = format!("{untitled_label} ({n})");
            n += 1;
            if n == u32::MAX {
                return;
            }
        }
        entry.fields.insert(name, value);
    }
}

fn unmarshal_field<E: InnerStream>(
    context: &mut XmlDecodeContext<E>,
    node: &Node,
) -> Result<(Option<String>, EntryValue), FormatError> {
    let key = node
        .first(format_xml::tags::entry::fields::ITEM_KEY)
        .and_then(NodeXmlExt::get_text)
        .map(ToOwned::to_owned);
    let value_node = node.first(format_xml::tags::entry::fields::ITEM_VALUE);
    let protected = value_node
        .and_then(|node| node.attribute_value(format_xml::attributes::PROTECTED))
        .is_some_and(|value| value.eq_ignore_ascii_case("true"));
    let protect_in_memory = value_node
        .and_then(|node| node.attribute_value(format_xml::attributes::PROTECTED_IN_MEM_PLAIN_XML))
        .is_some_and(|value| value.eq_ignore_ascii_case("true"));

    if protected || protect_in_memory {
        let bytes = value_node
            .map(NodeXmlExt::get_bytes)
            .transpose()
            .map_err(|error| FormatError::InvalidXml(error.to_string()))?
            .flatten()
            .unwrap_or_default();
        let salt = context
            .encryption
            .get_salt(bytes.len())
            .map_err(|error| FormatError::InvalidXml(error.to_string()))?;
        Ok((key, EntryValue::Encrypted(EncryptedValue::new(bytes, salt))))
    } else {
        let text = value_node
            .and_then(NodeXmlExt::get_text)
            .unwrap_or("")
            .to_owned();
        Ok((key, EntryValue::Plain(text)))
    }
}

pub fn marshal_entry<E: InnerStream>(
    entry: &Entry,
    context: &mut XmlEncodeContext<E>,
) -> Result<Node, FormatError> {
    let mut node = Node::new(format_xml::tags::entry::TAG_NAME);
    node.element(format_xml::tags::UUID).add_uuid(entry.uuid);
    node.element(format_xml::tags::entry::ICON_ID)
        .text(entry.icon.ordinal().to_string());
    if let Some(custom_icon_uuid) = entry.custom_icon_uuid {
        node.element(format_xml::tags::entry::CUSTOM_ICON_ID)
            .add_uuid(custom_icon_uuid);
    }
    let foreground = node.element(format_xml::tags::entry::FOREGROUND_COLOR);
    if let Some(value) = &entry.foreground_color {
        foreground.text(value);
    }
    let background = node.element(format_xml::tags::entry::BACKGROUND_COLOR);
    if let Some(value) = &entry.background_color {
        background.text(value);
    }
    node.element(format_xml::tags::entry::OVERRIDE_URL)
        .text(&entry.override_url);
    node.element(format_xml::tags::entry::TAGS)
        .text(entry.tags.join(consts::TAGS_SEPARATOR));

    if context.version().is_at_least(4, 1) {
        node.element(format_xml::tags::entry::QUALITY_CHECK)
            .add_boolean(entry.quality_check);
    }
    if context.version().is_at_least(4, 1) {
        if let Some(previous_parent_group) = entry.previous_parent_group {
            node.element(format_xml::tags::entry::PREVIOUS_PARENT_GROUP)
                .add_uuid(previous_parent_group);
        }
    }
    if let Some(times) = &entry.times {
        node.add_element(
            marshal_time_data(times, context)
                .map_err(|error| FormatError::InvalidXml(error.to_string()))?,
        );
    }
    for field in marshal_fields(context, &entry.fields)? {
        node.add_element(field);
    }
    for binary in &entry.binaries {
        node.add_element(marshal_binary_reference(binary, context)?);
    }
    if !entry.custom_data.is_empty() {
        node.add_element(
            marshal_custom_data(context, &entry.custom_data)
                .map_err(|error| FormatError::InvalidXml(error.to_string()))?,
        );
    }
    if let Some(auto_type) = &entry.auto_type {
        node.add_element(marshal_auto_type_data(auto_type));
    }
    if !entry.history.is_empty() {
        let history = node.element(format_xml::tags::entry::HISTORY);
        for item in &entry.history {
            history.add_element(marshal_entry(item, context)?);
        }
    }

    Ok(node)
}

fn marshal_fields<E: InnerStream>(
    context: &mut XmlEncodeContext<E>,
    fields: &EntryFields,
) -> Result<Vec<Node>, FormatError> {
    fields
        .iter()
        .map(|(key, value)| marshal_field(context, key, value))
        .collect()
}

fn marshal_field<E: InnerStream>(
    context: &mut XmlEncodeContext<E>,
    key: &str,
    value: &EntryValue,
) -> Result<Node, FormatError> {
    let mut node = Node::new(format_xml::tags::entry::fields::TAG_NAME);
    node.element(format_xml::tags::entry::fields::ITEM_KEY)
        .text(key);
    let value_node = node.element(format_xml::tags::entry::fields::ITEM_VALUE);
    let is_protected = matches!(value, EntryValue::Encrypted(_));
    let protected_in_memory = match &*context {
        XmlEncodeContext::Plain {
            memory_protection_flags,
            ..
        } => memory_protection_flags
            .iter()
            .any(|flag| flag.to_basic_field().key() == key),
        XmlEncodeContext::Encrypted { .. } => false,
    };

    match context {
        XmlEncodeContext::Encrypted {
            inner_encryption, ..
        } => {
            if is_protected {
                let encrypted_content = inner_encryption
                    .process_bytes(value.content().as_bytes())
                    .map_err(|error| FormatError::InvalidXml(error.to_string()))?;
                value_node.attribute(format_xml::attributes::PROTECTED, format_xml::values::TRUE);
                value_node.add_bytes(&encrypted_content);
            } else {
                value_node.text(value.content());
            }
        }
        XmlEncodeContext::Plain { .. } => {
            if is_protected || protected_in_memory {
                value_node.attribute(
                    format_xml::attributes::PROTECTED_IN_MEM_PLAIN_XML,
                    format_xml::values::TRUE,
                );
            }
            value_node.text(value.content());
        }
    }

    Ok(node)
}
