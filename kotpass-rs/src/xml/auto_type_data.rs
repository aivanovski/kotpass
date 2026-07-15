use crate::{
    constants::AutoTypeObfuscation,
    model::{AutoTypeData, AutoTypeItem},
    xml::{Node, NodeXmlExt, format_xml::tags},
};

pub fn unmarshal_auto_type_data(node: &Node) -> AutoTypeData {
    AutoTypeData {
        enabled: node
            .first(tags::entry::auto_type::ENABLED)
            .and_then(NodeXmlExt::get_text)
            .is_some_and(|text| text.eq_ignore_ascii_case("true")),
        obfuscation: node
            .first(tags::entry::auto_type::OBFUSCATION)
            .and_then(NodeXmlExt::get_text)
            .and_then(|text| text.parse::<usize>().ok())
            .and_then(AutoTypeObfuscation::from_ordinal)
            .unwrap_or(AutoTypeObfuscation::None),
        default_sequence: node
            .first(tags::entry::auto_type::DEFAULT_SEQUENCE)
            .and_then(NodeXmlExt::get_text)
            .map(ToOwned::to_owned),
        items: unmarshal_auto_type_items(node),
    }
}

fn unmarshal_auto_type_items(node: &Node) -> Vec<AutoTypeItem> {
    node.child_nodes()
        .into_iter()
        .filter(|node| node.node_name == tags::entry::auto_type::ASSOCIATION)
        .filter_map(|node| {
            let window = node
                .first(tags::entry::auto_type::WINDOW)
                .and_then(NodeXmlExt::get_text)?;
            let sequence = node
                .first(tags::entry::auto_type::KEYSTROKE_SEQUENCE)
                .and_then(NodeXmlExt::get_text)?;

            Some(AutoTypeItem::new(window, sequence))
        })
        .collect()
}

pub fn marshal_auto_type_data(data: &AutoTypeData) -> Node {
    let mut node = Node::new(tags::entry::auto_type::TAG_NAME);
    node.element(tags::entry::auto_type::ENABLED)
        .add_boolean(data.enabled);
    node.element(tags::entry::auto_type::OBFUSCATION)
        .text(data.obfuscation.ordinal().to_string());
    node.element(tags::entry::auto_type::DEFAULT_SEQUENCE)
        .text(data.default_sequence.as_deref().unwrap_or(""));

    for item in &data.items {
        let association = node.element(tags::entry::auto_type::ASSOCIATION);
        association
            .element(tags::entry::auto_type::WINDOW)
            .text(&item.window);
        association
            .element(tags::entry::auto_type::KEYSTROKE_SEQUENCE)
            .text(&item.keystroke_sequence);
    }

    node
}

#[cfg(test)]
mod tests {
    use super::{marshal_auto_type_data, unmarshal_auto_type_data};
    use crate::{
        constants::AutoTypeObfuscation,
        model::{AutoTypeData, AutoTypeItem},
        xml::{Node, NodeXmlExt, format_xml::tags},
    };

    fn sample_node() -> Node {
        let mut node = Node::new(tags::entry::auto_type::TAG_NAME);
        node.element(tags::entry::auto_type::ENABLED).text("True");
        node.element(tags::entry::auto_type::OBFUSCATION).text("1");
        node.element(tags::entry::auto_type::DEFAULT_SEQUENCE)
            .text("{USERNAME}{TAB}{PASSWORD}{ENTER}");
        let association = node.element(tags::entry::auto_type::ASSOCIATION);
        association.element(tags::entry::auto_type::WINDOW).text("*");
        association
            .element(tags::entry::auto_type::KEYSTROKE_SEQUENCE)
            .text("{PASSWORD}");
        node.element(tags::entry::auto_type::ASSOCIATION)
            .element(tags::entry::auto_type::WINDOW)
            .text("missing sequence");
        node
    }

    #[test]
    fn unmarshals_auto_type_data_like_kotlin_mapper() {
        let data = unmarshal_auto_type_data(&sample_node());

        assert!(data.enabled);
        assert_eq!(data.obfuscation, AutoTypeObfuscation::UseClipboard);
        assert_eq!(
            data.default_sequence.as_deref(),
            Some("{USERNAME}{TAB}{PASSWORD}{ENTER}")
        );
        assert_eq!(data.items, vec![AutoTypeItem::new("*", "{PASSWORD}")]);
    }

    #[test]
    fn defaults_missing_or_invalid_values_like_kotlin_mapper() {
        let mut node = Node::new(tags::entry::auto_type::TAG_NAME);
        node.element(tags::entry::auto_type::ENABLED).text("yes");
        node.element(tags::entry::auto_type::OBFUSCATION)
            .text("not an ordinal");

        let data = unmarshal_auto_type_data(&node);

        assert!(!data.enabled);
        assert_eq!(data.obfuscation, AutoTypeObfuscation::None);
        assert_eq!(data.default_sequence, None);
        assert!(data.items.is_empty());
    }

    #[test]
    fn marshals_auto_type_data_like_kotlin_mapper() {
        let data = AutoTypeData {
            enabled: true,
            obfuscation: AutoTypeObfuscation::UseClipboard,
            default_sequence: None,
            items: vec![AutoTypeItem::new("*", "{PASSWORD}")],
        };

        let node = marshal_auto_type_data(&data);

        assert_eq!(node.node_name, tags::entry::auto_type::TAG_NAME);
        assert_eq!(
            node.first(tags::entry::auto_type::ENABLED)
                .and_then(NodeXmlExt::get_text),
            Some("True")
        );
        assert_eq!(
            node.first(tags::entry::auto_type::OBFUSCATION)
                .and_then(NodeXmlExt::get_text),
            Some("1")
        );
        assert_eq!(
            node.first(tags::entry::auto_type::DEFAULT_SEQUENCE)
                .and_then(NodeXmlExt::get_text),
            Some("")
        );
        assert_eq!(
            node.first(tags::entry::auto_type::ASSOCIATION)
                .and_then(|node| node.first(tags::entry::auto_type::KEYSTROKE_SEQUENCE))
                .and_then(NodeXmlExt::get_text),
            Some("{PASSWORD}")
        );
    }
}
