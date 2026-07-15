use quick_xml::{
    Reader,
    escape::unescape,
    events::{Event, attributes::AttrError},
};

use super::{AttributeValue, Element, Namespace, Node, XmlVersion, build_name};

pub fn xml(
    root: impl AsRef<str>,
    encoding: Option<&str>,
    version: Option<XmlVersion>,
    namespace: Option<Namespace>,
) -> Node {
    let mut node = Node::new(build_name(root.as_ref(), namespace.as_ref()));
    if let Some(encoding) = encoding {
        node.set_encoding(encoding);
    }
    if let Some(version) = version {
        node.set_version(version);
    }
    if let Some(namespace) = namespace {
        node.add_namespace(namespace);
    }
    node
}

pub fn node(name: impl AsRef<str>, namespace: Option<Namespace>) -> Node {
    Node::new(build_name(name.as_ref(), namespace.as_ref()))
}

pub fn parse_str(input: &str) -> Result<Node, XmlParseError> {
    let mut reader = Reader::from_str(input);
    reader.config_mut().trim_text(true);
    let mut stack: Vec<Node> = Vec::new();
    let mut root = None;

    loop {
        match reader.read_event()? {
            Event::Start(event) => {
                let mut node = Node::new(name_to_string(event.name().as_ref())?);
                copy_attributes(&reader, event.attributes(), &mut node)?;
                stack.push(node);
            }
            Event::Empty(event) => {
                let mut node = Node::new(name_to_string(event.name().as_ref())?);
                copy_attributes(&reader, event.attributes(), &mut node)?;
                append_node(&mut stack, &mut root, node)?;
            }
            Event::End(_) => {
                let node = stack.pop().ok_or(XmlParseError::UnexpectedEnd)?;
                append_node(&mut stack, &mut root, node)?;
            }
            Event::Text(event) => {
                let text = event.xml10_content()?;
                let text = unescape(&text)?.trim().to_owned();
                if !text.is_empty()
                    && let Some(parent) = stack.last_mut()
                {
                    parent.text(text);
                }
            }
            Event::CData(event) => {
                let text = event.xml10_content()?.into_owned();
                if let Some(parent) = stack.last_mut() {
                    parent.cdata(text);
                }
            }
            Event::Eof => break,
            Event::Decl(_)
            | Event::PI(_)
            | Event::DocType(_)
            | Event::Comment(_)
            | Event::GeneralRef(_) => {}
        }
    }

    root.ok_or(XmlParseError::MissingRoot)
}

fn append_node(
    stack: &mut [Node],
    root: &mut Option<Node>,
    node: Node,
) -> Result<(), XmlParseError> {
    if let Some(parent) = stack.last_mut() {
        parent.add_element(Element::from(node));
    } else if root.is_none() {
        *root = Some(node);
    } else {
        return Err(XmlParseError::MultipleRoots);
    }
    Ok(())
}

fn copy_attributes<'a>(
    reader: &Reader<&[u8]>,
    attributes: quick_xml::events::attributes::Attributes<'a>,
    node: &mut Node,
) -> Result<(), XmlParseError> {
    for attribute in attributes {
        let attribute = attribute?;
        let name = name_to_string(attribute.key.as_ref())?;
        let value = attribute
            .decoded_and_normalized_value(
                quick_xml::XmlVersion::Implicit1_0,
                reader.decoder(),
            )?
            .into_owned();

        if let Some(namespace_name) = name.strip_prefix("xmlns") {
            let namespace_name = namespace_name.strip_prefix(':').unwrap_or(namespace_name);
            node.add_namespace(Namespace::new(namespace_name, value));
        } else {
            node.attribute(name, AttributeValue::Safe(value));
        }
    }
    Ok(())
}

fn name_to_string(name: &[u8]) -> Result<String, XmlParseError> {
    Ok(std::str::from_utf8(name)?.to_owned())
}

#[derive(Debug, thiserror::Error)]
pub enum XmlParseError {
    #[error(transparent)]
    QuickXml(#[from] quick_xml::Error),

    #[error(transparent)]
    Attribute(#[from] AttrError),

    #[error(transparent)]
    Encoding(#[from] quick_xml::encoding::EncodingError),

    #[error(transparent)]
    Escape(#[from] quick_xml::escape::EscapeError),

    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),

    #[error("unexpected end tag")]
    UnexpectedEnd,

    #[error("XML document has multiple roots")]
    MultipleRoots,

    #[error("XML document has no root")]
    MissingRoot,
}

#[cfg(test)]
mod tests {
    use super::{parse_str, xml};
    use crate::xml::{Element, Namespace, Node, PrintOptions, XmlVersion, unsafe_value};

    #[test]
    fn renders_basic_tree_like_kotlin_builder() {
        let mut root = xml("urlset", None, None, None);
        root.set_xmlns(Some(
            "https://www.sitemaps.org/schemas/sitemap/0.9".to_owned(),
        ));
        for index in 0..=2 {
            let url = root.element("url");
            url.element_text("loc", format!("https://google.com/{index}"));
        }

        assert_eq!(
            root.to_string_with_options(PrintOptions::default()),
            concat!(
                "<urlset xmlns=\"https://www.sitemaps.org/schemas/sitemap/0.9\">\n",
                "\t<url>\n",
                "\t\t<loc>\n",
                "\t\t\thttps://google.com/0\n",
                "\t\t</loc>\n",
                "\t</url>\n",
                "\t<url>\n",
                "\t\t<loc>\n",
                "\t\t\thttps://google.com/1\n",
                "\t\t</loc>\n",
                "\t</url>\n",
                "\t<url>\n",
                "\t\t<loc>\n",
                "\t\t\thttps://google.com/2\n",
                "\t\t</loc>\n",
                "\t</url>\n",
                "</urlset>"
            )
        );
    }

    #[test]
    fn renders_single_line_text_and_escaped_attributes() {
        let mut root = Node::new("root");
        root.attribute("attr", "& < > \" '");
        root.element_text("name", "value");

        assert_eq!(
            root.to_string_with_options(PrintOptions {
                single_line_text_elements: true,
                ..PrintOptions::default()
            }),
            "<root attr=\"&amp; &lt; &gt; &quot; &apos;\">\n\t<name>value</name>\n</root>"
        );
    }

    #[test]
    fn supports_character_references_and_unsafe_values() {
        let mut root = Node::new("root");
        root.attribute("safe", "&\"'");
        root.attribute("raw", unsafe_value("&\"'"));

        assert_eq!(
            root.to_string_with_options(PrintOptions {
                use_character_reference: true,
                ..PrintOptions::default()
            }),
            "<root safe=\"&#38;&#34;&#39;\" raw=\"&\"'\"/>"
        );
    }

    #[test]
    fn renders_cdata_comment_and_processing_instruction() {
        let mut root = Node::new("root");
        root.cdata("<![CDATA[Some & xml]]>");
        root.comment("my comment -->");
        root.processing_instruction(
            "xml-stylesheet",
            vec![("href".to_owned(), "http://blah".to_owned())],
        );

        assert_eq!(
            root.to_string_with_options(PrintOptions::default()),
            concat!(
                "<root>\n",
                "\t<![CDATA[<![CDATA[Some & xml]]]]><![CDATA[>]]>\n",
                "\t<!-- my comment &#45;&#45;> -->\n",
                "\t<?xml-stylesheet href=\"http://blah\"?>\n",
                "</root>"
            )
        );
    }

    #[test]
    fn suppresses_namespaces_declared_on_ancestors() {
        let namespace = Namespace::new("a", "https://ns1.org");
        let mut root = xml("root", None, None, Some(namespace.clone()));
        root.element_ns("child", namespace);

        assert_eq!(
            root.to_string_with_options(PrintOptions::default()),
            "<a:root xmlns:a=\"https://ns1.org\">\n\t<a:child/>\n</a:root>"
        );
    }

    #[test]
    fn supports_child_mutation() {
        let mut root = Node::new("root");
        root.add_element(Node::new("a"));
        let a = root.children()[0].clone();
        root.add_element_after(Node::new("b"), &a).unwrap();
        root.replace_element(&a, Node::new("c")).unwrap();

        assert_eq!(
            root.to_string_with_options(PrintOptions::default()),
            "<root>\n\t<c/>\n\t<b/>\n</root>"
        );
        assert!(matches!(root.children()[0], Element::Node(_)));
    }

    #[test]
    fn parses_elements_attributes_text_and_cdata() {
        let root = parse_str(
            "<root xmlns=\"https://blog.redundent.org\"><child key=\"a&amp;b\">value</child><![CDATA[x&y]]></root>",
        )
        .unwrap();

        assert_eq!(root.node_name, "root");
        assert_eq!(root.xmlns(), Some("https://blog.redundent.org"));
        let child = root.first("child").unwrap();
        assert_eq!(child.attribute_value("key"), Some("a&b"));
        assert_eq!(
            root.to_string_with_options(PrintOptions {
                single_line_text_elements: true,
                ..PrintOptions::default()
            }),
            "<root xmlns=\"https://blog.redundent.org\">\n\t<child key=\"a&amp;b\">value</child>\n\t<![CDATA[x&y]]>\n</root>"
        );
    }

    #[test]
    fn renders_xml_prolog_options() {
        let mut root = Node::new("root");
        root.set_encoding("utf-8");
        root.set_version(XmlVersion::V11);
        root.set_standalone(Some(true));

        assert_eq!(
            root.to_string_with_options(PrintOptions::default()),
            "<?xml version=\"1.1\" encoding=\"utf-8\" standalone=\"yes\"?>\n<root/>"
        );
    }
}
