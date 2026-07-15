use indexmap::IndexMap;

use crate::{
    error::FormatError,
    model::BinaryData,
    xml::{Node, NodeXmlExt, bool_to_xml_string, format_xml},
};

pub fn unmarshal_binaries(node: &Node) -> Result<IndexMap<Vec<u8>, BinaryData>, FormatError> {
    let mut binaries = node
        .child_nodes()
        .into_iter()
        .filter(|node| node.node_name == format_xml::tags::meta::binaries::ITEM)
        .map(unmarshal_binary_data)
        .collect::<Result<Vec<_>, _>>()?;

    binaries.sort_by_key(|(id, _)| *id);

    Ok(binaries
        .into_iter()
        .map(|(_, binary)| (binary.hash().to_vec(), binary))
        .collect())
}

fn unmarshal_binary_data(node: &Node) -> Result<(i32, BinaryData), FormatError> {
    let id = node
        .attribute_value(format_xml::attributes::ID)
        .ok_or_else(|| FormatError::InvalidXml("Binary node has no id.".to_owned()))?
        .parse::<i32>()
        .map_err(|_| FormatError::InvalidXml("Binary node has invalid id.".to_owned()))?;
    let bytes = node
        .get_bytes()
        .map_err(|error| FormatError::InvalidXml(error.to_string()))?
        .ok_or_else(|| FormatError::InvalidXml(format!("Empty body of binary node with id: {id}.")))?;
    let compressed = node
        .attribute_value(format_xml::attributes::COMPRESSED)
        .is_some_and(|value| value.eq_ignore_ascii_case("true"));
    let binary = if compressed {
        BinaryData::compressed(false, bytes)
    } else {
        BinaryData::uncompressed(false, bytes)
    };

    Ok((id, binary))
}

pub fn marshal_binary_data(binary: &BinaryData, id: i32) -> Node {
    let compressed = matches!(binary, BinaryData::Compressed { .. });
    let mut node = Node::new(format_xml::tags::meta::binaries::ITEM);
    node.attribute(format_xml::attributes::ID, id.to_string());
    node.attribute(
        format_xml::attributes::COMPRESSED,
        bool_to_xml_string(compressed),
    );
    node.add_bytes(binary.raw_content());
    node
}

#[cfg(test)]
mod tests {
    use super::{marshal_binary_data, unmarshal_binaries};
    use crate::{
        error::FormatError,
        model::BinaryData,
        xml::{Node, NodeXmlExt, format_xml},
    };

    fn binary_node(id: i32, compressed: bool, bytes: &[u8]) -> Node {
        let binary = if compressed {
            BinaryData::compressed(false, bytes.to_vec())
        } else {
            BinaryData::uncompressed(false, bytes.to_vec())
        };
        marshal_binary_data(&binary, id)
    }

    #[test]
    fn unmarshals_binaries_sorted_by_id_and_keyed_by_hash() {
        let mut root = Node::new(format_xml::tags::meta::binaries::TAG_NAME);
        root.add_element(binary_node(2, true, b"two"));
        root.add_element(binary_node(1, false, b"one"));

        let binaries = unmarshal_binaries(&root).unwrap();
        let raw_contents = binaries
            .values()
            .map(|binary| binary.raw_content().to_vec())
            .collect::<Vec<_>>();

        assert_eq!(raw_contents, vec![b"one".to_vec(), b"two".to_vec()]);
        assert!(matches!(
            binaries.values().nth(1).unwrap(),
            BinaryData::Compressed { .. }
        ));
        assert!(!binaries.values().next().unwrap().memory_protection());
    }

    #[test]
    fn rejects_missing_binary_id() {
        let mut root = Node::new(format_xml::tags::meta::binaries::TAG_NAME);
        let mut binary = Node::new(format_xml::tags::meta::binaries::ITEM);
        binary.add_bytes(b"data");
        root.add_element(binary);

        assert_eq!(
            unmarshal_binaries(&root),
            Err(FormatError::InvalidXml("Binary node has no id.".to_owned()))
        );
    }

    #[test]
    fn rejects_missing_binary_body() {
        let mut root = Node::new(format_xml::tags::meta::binaries::TAG_NAME);
        let mut binary = Node::new(format_xml::tags::meta::binaries::ITEM);
        binary.attribute(format_xml::attributes::ID, "7");
        root.add_element(binary);

        assert_eq!(
            unmarshal_binaries(&root),
            Err(FormatError::InvalidXml(
                "Empty body of binary node with id: 7.".to_owned()
            ))
        );
    }

    #[test]
    fn marshals_binary_data_like_kotlin_mapper() {
        let binary = BinaryData::compressed(false, b"data".to_vec());
        let node = marshal_binary_data(&binary, 42);

        assert_eq!(node.node_name, format_xml::tags::meta::binaries::ITEM);
        assert_eq!(node.attribute_value(format_xml::attributes::ID), Some("42"));
        assert_eq!(
            node.attribute_value(format_xml::attributes::COMPRESSED),
            Some("True")
        );
        assert_eq!(node.get_bytes().unwrap(), Some(b"data".to_vec()));
    }
}
