use crate::{
    error::FormatError,
    model::{DeletedObject, XmlEncodeContext},
    xml::{Node, NodeInstantExt, NodeXmlExt, XmlInstantError, format_xml},
};

pub fn unmarshal_deleted_object(node: &Node) -> Result<Option<DeletedObject>, FormatError> {
    let uuid = node
        .first(format_xml::tags::UUID)
        .map(NodeXmlExt::get_uuid)
        .transpose()
        .map_err(|error| FormatError::InvalidXml(error.to_string()))?
        .flatten();
    let deletion_time = node
        .first(format_xml::tags::deleted_objects::TIME)
        .map(NodeInstantExt::get_instant)
        .transpose()
        .map_err(|error| FormatError::InvalidXml(error.to_string()))?
        .flatten();

    Ok(match (uuid, deletion_time) {
        (Some(id), Some(deletion_time)) => Some(DeletedObject::new(id, deletion_time)),
        _ => None,
    })
}

pub fn marshal_deleted_object<E>(
    deleted_object: &DeletedObject,
    context: &XmlEncodeContext<E>,
) -> Result<Node, XmlInstantError> {
    let mut node = Node::new(format_xml::tags::deleted_objects::OBJECT);
    node.element(format_xml::tags::UUID)
        .add_uuid(deleted_object.id);
    node.element(format_xml::tags::deleted_objects::TIME)
        .add_date_time(context, Some(deleted_object.deletion_time))?;
    Ok(node)
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};
    use uuid::Uuid;

    use super::{marshal_deleted_object, unmarshal_deleted_object};
    use crate::{
        model::{BinaryData, DeletedObject, FormatVersion, XmlEncodeContext},
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
    fn unmarshals_deleted_object_when_both_fields_exist() {
        let id = Uuid::parse_str("12345678-9abc-def0-1234-56789abcdef0").unwrap();
        let mut node = Node::new(format_xml::tags::deleted_objects::OBJECT);
        node.element(format_xml::tags::UUID).add_uuid(id);
        node.element(format_xml::tags::deleted_objects::TIME)
            .text("2020-01-02T03:04:05Z");

        let deleted = unmarshal_deleted_object(&node).unwrap().unwrap();

        assert_eq!(deleted.id, id);
        assert_eq!(deleted.deletion_time.unix_timestamp(), 1_577_934_245);
    }

    #[test]
    fn skips_deleted_object_when_a_field_is_missing() {
        let mut node = Node::new(format_xml::tags::deleted_objects::OBJECT);
        node.element(format_xml::tags::UUID)
            .add_uuid(Uuid::from_u128(1));

        assert_eq!(unmarshal_deleted_object(&node).unwrap(), None);
    }

    #[test]
    fn marshals_deleted_object_like_kotlin_mapper() {
        let id = Uuid::parse_str("12345678-9abc-def0-1234-56789abcdef0").unwrap();
        let deletion_time = OffsetDateTime::parse("2020-01-02T03:04:05Z", &Rfc3339).unwrap();
        let deleted = DeletedObject::new(id, deletion_time);

        let node = marshal_deleted_object(&deleted, &context()).unwrap();

        assert_eq!(node.node_name, format_xml::tags::deleted_objects::OBJECT);
        assert_eq!(
            node.first(format_xml::tags::UUID)
                .and_then(NodeXmlExt::get_text),
            Some("EjRWeJq83vASNFZ4mrze8A==")
        );
        assert_eq!(
            node.first(format_xml::tags::deleted_objects::TIME)
                .and_then(NodeXmlExt::get_text),
            Some("2020-01-02T03:04:05Z")
        );
    }
}
