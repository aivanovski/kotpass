use uuid::Uuid;

use crate::{
    constants::GroupOverride,
    io::base64::{Base64Error, decode_base64_to_array, encode_base64},
    xml::{Element, Node},
};

pub trait NodeXmlExt {
    fn child_nodes(&self) -> Vec<&Node>;
    fn get_text(&self) -> Option<&str>;
    fn get_group_override(&self) -> GroupOverride;
    fn get_uuid(&self) -> Result<Option<Uuid>, NodeValueError>;
    fn get_bytes(&self) -> Result<Option<Vec<u8>>, Base64Error>;
    fn add_boolean(&mut self, value: bool);
    fn add_group_override(&mut self, value: GroupOverride);
    fn add_uuid(&mut self, value: Uuid);
    fn add_bytes(&mut self, bytes: &[u8]);
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NodeValueError {
    #[error(transparent)]
    Base64(#[from] Base64Error),

    #[error("UUID value must decode to 16 bytes, got {0}")]
    InvalidUuidLength(usize),
}

impl NodeXmlExt for Node {
    fn child_nodes(&self) -> Vec<&Node> {
        self.children()
            .iter()
            .filter_map(Element::as_node)
            .collect()
    }

    fn get_text(&self) -> Option<&str> {
        self.children().first().and_then(|element| match element {
            Element::Text(text) => Some(text.text.as_str()),
            _ => None,
        })
    }

    fn get_group_override(&self) -> GroupOverride {
        let value = self
            .get_text()
            .and_then(|text| match text.to_ascii_lowercase().as_str() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            });

        GroupOverride::from_bool(value)
    }

    fn get_uuid(&self) -> Result<Option<Uuid>, NodeValueError> {
        let Some(text) = self.get_text() else {
            return Ok(None);
        };
        let bytes = decode_base64_to_array(text)?;
        let length = bytes.len();
        let bytes: [u8; 16] = bytes
            .try_into()
            .map_err(|_| NodeValueError::InvalidUuidLength(length))?;
        Ok(Some(Uuid::from_bytes(bytes)))
    }

    fn get_bytes(&self) -> Result<Option<Vec<u8>>, Base64Error> {
        self.get_text().map(decode_base64_to_array).transpose()
    }

    fn add_boolean(&mut self, value: bool) {
        self.text(crate::xml::bool_to_xml_string(value));
    }

    fn add_group_override(&mut self, value: GroupOverride) {
        self.text(value.xml_value());
    }

    fn add_uuid(&mut self, value: Uuid) {
        self.text(encode_base64(value.as_bytes()));
    }

    fn add_bytes(&mut self, bytes: &[u8]) {
        self.text(encode_base64(bytes));
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::{NodeValueError, NodeXmlExt};
    use crate::{
        constants::GroupOverride,
        io::base64::Base64Error,
        xml::{Element, Node},
    };

    #[test]
    fn returns_child_nodes_and_first_text_child_like_kotlin_extensions() {
        let mut root = Node::new("Root");
        root.text("hello");
        root.comment("ignored");
        root.element("Child");

        assert_eq!(root.get_text(), Some("hello"));
        assert_eq!(root.child_nodes().len(), 1);
        assert_eq!(root.child_nodes()[0].node_name, "Child");
    }

    #[test]
    fn ignores_non_text_first_child_for_text_lookup() {
        let mut root = Node::new("Root");
        root.add_element(Element::from(Node::new("Child")));
        root.text("not first");

        assert_eq!(root.get_text(), None);
    }

    #[test]
    fn parses_group_override_values_case_insensitively() {
        let mut enabled = Node::new("Enabled");
        enabled.text("TRUE");
        let mut disabled = Node::new("Disabled");
        disabled.text("false");
        let mut inherit = Node::new("Inherit");
        inherit.text("Null");

        assert_eq!(enabled.get_group_override(), GroupOverride::Enabled);
        assert_eq!(disabled.get_group_override(), GroupOverride::Disabled);
        assert_eq!(inherit.get_group_override(), GroupOverride::Inherit);
        assert_eq!(
            Node::new("Missing").get_group_override(),
            GroupOverride::Inherit
        );
    }

    #[test]
    fn reads_and_writes_uuid_as_base64_bytes() {
        let uuid = Uuid::parse_str("12345678-9abc-def0-1234-56789abcdef0").unwrap();
        let mut node = Node::new("UUID");
        node.add_uuid(uuid);

        assert_eq!(node.get_text(), Some("EjRWeJq83vASNFZ4mrze8A=="));
        assert_eq!(node.get_uuid().unwrap(), Some(uuid));
    }

    #[test]
    fn rejects_base64_uuid_with_wrong_length() {
        let mut node = Node::new("UUID");
        node.text("AA==");

        assert_eq!(node.get_uuid(), Err(NodeValueError::InvalidUuidLength(1)));
    }

    #[test]
    fn reads_and_writes_byte_values_as_base64() {
        let mut node = Node::new("Data");
        node.add_bytes(&[1, 2, 3, 4]);

        assert_eq!(node.get_text(), Some("AQIDBA=="));
        assert_eq!(node.get_bytes().unwrap(), Some(vec![1, 2, 3, 4]));
    }

    #[test]
    fn preserves_base64_decode_errors_for_byte_values() {
        let mut node = Node::new("Data");
        node.text("?");

        assert_eq!(node.get_bytes(), Err(Base64Error::UnexpectedCharacter(0)));
    }

    #[test]
    fn writes_boolean_and_group_override_literals() {
        let mut boolean = Node::new("Boolean");
        boolean.add_boolean(true);
        let mut override_node = Node::new("Override");
        override_node.add_group_override(GroupOverride::Disabled);

        assert_eq!(boolean.get_text(), Some("True"));
        assert_eq!(override_node.get_text(), Some("False"));
    }
}
