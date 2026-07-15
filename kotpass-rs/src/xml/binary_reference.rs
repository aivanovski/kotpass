use crate::{
    error::FormatError,
    io::base16::encode_hex,
    model::{BinaryReference, XmlDecodeContext, XmlEncodeContext},
    xml::{Node, NodeXmlExt, format_xml},
};

pub fn unmarshal_binary_reference<E>(
    context: &XmlDecodeContext<E>,
    node: &Node,
) -> Result<Option<BinaryReference>, FormatError> {
    let id = node
        .first(format_xml::tags::entry::binary_references::ITEM_VALUE)
        .and_then(|node| node.attribute_value(format_xml::attributes::REF))
        .ok_or_else(|| FormatError::InvalidXml("Invalid binary reference id.".to_owned()))?
        .parse::<usize>()
        .map_err(|_| FormatError::InvalidXml("Invalid binary reference id.".to_owned()))?;
    let Some(hash) = context.binaries.keys().nth(id) else {
        return Ok(None);
    };
    let name = node
        .first(format_xml::tags::entry::binary_references::ITEM_KEY)
        .and_then(NodeXmlExt::get_text)
        .ok_or_else(|| FormatError::InvalidXml("Invalid binary reference key.".to_owned()))?;

    Ok(Some(BinaryReference::new(hash.clone(), name)))
}

pub fn marshal_binary_reference<E>(
    reference: &BinaryReference,
    context: &XmlEncodeContext<E>,
) -> Result<Node, FormatError> {
    let id = context
        .binaries()
        .keys()
        .position(|hash| hash == &reference.hash)
        .ok_or_else(|| {
            FormatError::InvalidContent(format!(
                "No binary with hash: {}.",
                encode_hex(&reference.hash)
            ))
        })?;

    let mut node = Node::new(format_xml::tags::entry::binary_references::TAG_NAME);
    node.element(format_xml::tags::entry::binary_references::ITEM_KEY)
        .text(&reference.name);
    node.element(format_xml::tags::entry::binary_references::ITEM_VALUE)
        .attribute(format_xml::attributes::REF, id.to_string());
    Ok(node)
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use super::{marshal_binary_reference, unmarshal_binary_reference};
    use crate::{
        error::FormatError,
        model::{BinaryData, BinaryReference, FormatVersion, XmlDecodeContext, XmlEncodeContext},
        xml::{Node, NodeXmlExt, format_xml},
    };

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct NoEncryption;

    fn binaries() -> IndexMap<Vec<u8>, BinaryData> {
        let one = BinaryData::uncompressed(false, b"one".to_vec());
        let two = BinaryData::uncompressed(false, b"two".to_vec());
        [(one.hash().to_vec(), one), (two.hash().to_vec(), two)]
            .into_iter()
            .collect()
    }

    fn decode_context() -> XmlDecodeContext<NoEncryption> {
        XmlDecodeContext::new(FormatVersion::new(4, 1), NoEncryption, binaries())
    }

    fn encode_context() -> XmlEncodeContext<NoEncryption> {
        XmlEncodeContext::Plain {
            version: FormatVersion::new(4, 1),
            binaries: binaries(),
            memory_protection_flags: Default::default(),
        }
    }

    fn reference_node(ref_id: &str, key: Option<&str>) -> Node {
        let mut node = Node::new(format_xml::tags::entry::binary_references::TAG_NAME);
        if let Some(key) = key {
            node.element(format_xml::tags::entry::binary_references::ITEM_KEY)
                .text(key);
        }
        node.element(format_xml::tags::entry::binary_references::ITEM_VALUE)
            .attribute(format_xml::attributes::REF, ref_id);
        node
    }

    #[test]
    fn unmarshals_binary_reference_from_context_index() {
        let reference = unmarshal_binary_reference(&decode_context(), &reference_node("1", Some("file.txt")))
            .unwrap()
            .unwrap();

        assert_eq!(reference.name, "file.txt");
        assert_eq!(
            reference.hash,
            decode_context().binaries.keys().nth(1).unwrap().clone()
        );
    }

    #[test]
    fn drops_reference_with_out_of_range_id() {
        assert_eq!(
            unmarshal_binary_reference(&decode_context(), &reference_node("9", Some("file.txt")))
                .unwrap(),
            None
        );
    }

    #[test]
    fn rejects_malformed_binary_reference() {
        assert_eq!(
            unmarshal_binary_reference(&decode_context(), &reference_node("abc", Some("file.txt"))),
            Err(FormatError::InvalidXml(
                "Invalid binary reference id.".to_owned()
            ))
        );
        assert_eq!(
            unmarshal_binary_reference(&decode_context(), &reference_node("0", None)),
            Err(FormatError::InvalidXml(
                "Invalid binary reference key.".to_owned()
            ))
        );
    }

    #[test]
    fn marshals_binary_reference_to_context_index() {
        let context = encode_context();
        let hash = context.binaries().keys().nth(1).unwrap().clone();
        let reference = BinaryReference::new(hash, "file.txt");

        let node = marshal_binary_reference(&reference, &context).unwrap();

        assert_eq!(
            node.first(format_xml::tags::entry::binary_references::ITEM_KEY)
                .and_then(NodeXmlExt::get_text),
            Some("file.txt")
        );
        assert_eq!(
            node.first(format_xml::tags::entry::binary_references::ITEM_VALUE)
                .and_then(|node| node.attribute_value(format_xml::attributes::REF)),
            Some("1")
        );
    }

    #[test]
    fn rejects_reference_to_missing_binary_hash() {
        let reference = BinaryReference::new(vec![1, 2, 3], "missing");

        assert_eq!(
            marshal_binary_reference(&reference, &encode_context()),
            Err(FormatError::InvalidContent(
                "No binary with hash: 010203.".to_owned()
            ))
        );
    }
}
