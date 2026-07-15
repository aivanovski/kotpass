use indexmap::{IndexMap, IndexSet};

use super::{
    Attribute, AttributeValue, CDataElement, Comment, Doctype, Element, Namespace, PrintOptions,
    ProcessingInstructionElement, TextElement, XmlVersion, build_name, escape_value,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub node_name: String,
    global_processing_instructions: Vec<ProcessingInstructionElement>,
    doctype: Option<Doctype>,
    namespaces: IndexSet<Namespace>,
    attributes: IndexMap<String, AttributeValue>,
    children: Vec<Element>,
    pub include_xml_prolog: bool,
    pub encoding: String,
    pub version: XmlVersion,
    pub standalone: Option<bool>,
}

impl Node {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            node_name: name.into(),
            global_processing_instructions: Vec::new(),
            doctype: None,
            namespaces: IndexSet::new(),
            attributes: IndexMap::new(),
            children: Vec::new(),
            include_xml_prolog: false,
            encoding: "UTF-8".to_owned(),
            version: XmlVersion::V10,
            standalone: None,
        }
    }

    pub fn set_encoding(&mut self, encoding: impl Into<String>) {
        self.include_xml_prolog = true;
        self.encoding = encoding.into();
    }

    pub fn set_version(&mut self, version: XmlVersion) {
        self.include_xml_prolog = true;
        self.version = version;
    }

    pub fn set_standalone(&mut self, standalone: Option<bool>) {
        self.include_xml_prolog = true;
        self.standalone = standalone;
    }

    pub fn namespaces(&self) -> &IndexSet<Namespace> {
        &self.namespaces
    }

    pub fn xmlns(&self) -> Option<&str> {
        self.namespaces
            .iter()
            .find(|namespace| namespace.is_default())
            .map(|namespace| namespace.value.as_str())
    }

    pub fn set_xmlns(&mut self, value: Option<String>) {
        self.namespaces.retain(|namespace| !namespace.is_default());
        if let Some(value) = value {
            self.add_namespace(Namespace::default(value));
        }
    }

    pub fn attributes(&self) -> &IndexMap<String, AttributeValue> {
        &self.attributes
    }

    pub fn attribute_value(&self, name: &str) -> Option<&str> {
        self.attributes.get(name).and_then(AttributeValue::as_deref)
    }

    pub fn children(&self) -> &[Element] {
        &self.children
    }

    pub fn set_attribute(&mut self, name: impl Into<String>, value: Option<AttributeValue>) {
        let name = name.into();
        if let Some(value) = value {
            self.attributes.insert(name, value);
        } else {
            self.attributes.shift_remove(&name);
        }
    }

    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }

    pub fn remove_attribute(&mut self, name: &str) -> Option<AttributeValue> {
        self.attributes.shift_remove(name)
    }

    pub fn text(&mut self, text: impl Into<String>) {
        self.children.push(TextElement::new(text).into());
    }

    pub fn unsafe_text(&mut self, text: impl Into<String>) {
        self.children.push(TextElement::unsafe_text(text).into());
    }

    pub fn comment(&mut self, text: impl Into<String>) {
        self.children.push(Comment::new(text).into());
    }

    pub fn element(&mut self, name: impl AsRef<str>) -> &mut Node {
        self.add_element(Node::new(name.as_ref()));
        self.children
            .last_mut()
            .and_then(|element| match element {
                Element::Node(node) => Some(node.as_mut()),
                _ => None,
            })
            .expect("just inserted node")
    }

    pub fn element_ns(&mut self, name: impl AsRef<str>, namespace: Namespace) -> &mut Node {
        let mut node = Node::new(build_name(name.as_ref(), Some(&namespace)));
        node.add_namespace(namespace);
        self.add_element(node);
        self.children
            .last_mut()
            .and_then(|element| match element {
                Element::Node(node) => Some(node.as_mut()),
                _ => None,
            })
            .expect("just inserted node")
    }

    pub fn element_text(&mut self, name: impl AsRef<str>, value: impl Into<String>) -> &mut Node {
        let node = self.element(name);
        node.text(value);
        node
    }

    pub fn attribute(&mut self, name: impl AsRef<str>, value: impl Into<AttributeValue>) {
        self.attributes
            .insert(name.as_ref().to_owned(), value.into());
    }

    pub fn attribute_ns(
        &mut self,
        name: impl AsRef<str>,
        value: impl Into<AttributeValue>,
        namespace: Namespace,
    ) {
        self.add_namespace(namespace.clone());
        self.attributes
            .insert(build_name(name.as_ref(), Some(&namespace)), value.into());
    }

    pub fn add_attribute(&mut self, attribute: Attribute) {
        if let Some(namespace) = attribute.namespace.clone() {
            self.attribute_ns(attribute.name, attribute.value, namespace);
        } else {
            self.attribute(attribute.name, attribute.value);
        }
    }

    pub fn cdata(&mut self, text: impl Into<String>) {
        self.children.push(CDataElement::new(text).into());
    }

    pub fn processing_instruction(
        &mut self,
        text: impl Into<String>,
        attributes: Vec<(String, String)>,
    ) {
        self.children
            .push(ProcessingInstructionElement::new(text, attributes).into());
    }

    pub fn global_processing_instruction(
        &mut self,
        text: impl Into<String>,
        attributes: Vec<(String, String)>,
    ) {
        self.global_processing_instructions
            .push(ProcessingInstructionElement::new(text, attributes));
    }

    pub fn doctype(
        &mut self,
        name: Option<String>,
        public_id: Option<String>,
        system_id: Option<String>,
    ) -> Result<(), &'static str> {
        if public_id.is_some() && system_id.is_none() {
            return Err("system_id must be provided if public_id is provided");
        }

        self.doctype = Some(Doctype::new(
            name.unwrap_or_else(|| self.node_name.clone()),
            public_id,
            system_id,
        ));
        Ok(())
    }

    pub fn namespace(&mut self, name: impl Into<String>, value: impl Into<String>) -> Namespace {
        let namespace = Namespace::new(name, value);
        self.add_namespace(namespace.clone());
        namespace
    }

    pub fn add_namespace(&mut self, namespace: Namespace) {
        if namespace.is_default() {
            self.namespaces.retain(|candidate| !candidate.is_default());
        }
        self.namespaces.insert(namespace);
    }

    pub fn add_element(&mut self, element: impl Into<Element>) {
        self.children.push(element.into());
    }

    pub fn add_elements(&mut self, elements: impl IntoIterator<Item = Element>) {
        self.children.extend(elements);
    }

    pub fn add_element_after(
        &mut self,
        element: impl Into<Element>,
        after: &Element,
    ) -> Result<(), String> {
        let index = self.find_index(after)?;
        self.children.insert(index + 1, element.into());
        Ok(())
    }

    pub fn add_element_before(
        &mut self,
        element: impl Into<Element>,
        before: &Element,
    ) -> Result<(), String> {
        let index = self.find_index(before)?;
        self.children.insert(index, element.into());
        Ok(())
    }

    pub fn remove_element(&mut self, element: &Element) -> Result<Element, String> {
        let index = self.find_index(element)?;
        Ok(self.children.remove(index))
    }

    pub fn replace_element(
        &mut self,
        existing: &Element,
        new_element: impl Into<Element>,
    ) -> Result<(), String> {
        let index = self.find_index(existing)?;
        self.children[index] = new_element.into();
        Ok(())
    }

    pub fn filter(&self, name: &str) -> Vec<&Node> {
        self.filter_by(|node| node.node_name == name)
    }

    pub fn filter_by(&self, predicate: impl Fn(&Node) -> bool) -> Vec<&Node> {
        self.children
            .iter()
            .filter_map(Element::as_node)
            .filter(|node| predicate(node))
            .collect()
    }

    pub fn first(&self, name: &str) -> Option<&Node> {
        self.children
            .iter()
            .filter_map(Element::as_node)
            .find(|node| node.node_name == name)
    }

    pub fn exists(&self, name: &str) -> bool {
        self.first(name).is_some()
    }

    pub fn to_string_pretty(&self, pretty: bool) -> String {
        self.to_string_with_options(PrintOptions {
            pretty,
            ..PrintOptions::default()
        })
    }

    pub fn to_string_with_options(&self, mut print_options: PrintOptions) -> String {
        let mut output = String::new();
        self.write_to(&mut output, &mut print_options);
        output.trim().to_owned()
    }

    pub fn write_to(&self, builder: &mut String, print_options: &mut PrintOptions) {
        print_options.xml_version = self.version;

        if self.include_xml_prolog {
            builder.push_str("<?xml version=\"");
            builder.push_str(print_options.xml_version.value());
            builder.push_str("\" encoding=\"");
            builder.push_str(&self.encoding);
            builder.push('"');

            if let Some(standalone) = self.standalone {
                builder.push_str(" standalone=\"");
                builder.push_str(if standalone { "yes" } else { "no" });
                builder.push('"');
            }

            builder.push_str("?>");
            builder.push_str(print_options.line_ending());
        }

        if let Some(doctype) = &self.doctype {
            doctype.render(builder, print_options);
        }

        for instruction in &self.global_processing_instructions {
            instruction.render(builder, "", print_options);
        }

        self.render(builder, "", print_options, &[]);
    }

    pub(crate) fn render(
        &self,
        builder: &mut String,
        indent: &str,
        print_options: &PrintOptions,
        parent_namespaces: &[Namespace],
    ) {
        let line_ending = print_options.line_ending();
        builder.push_str(indent);
        builder.push('<');
        builder.push_str(&self.node_name);
        self.render_namespaces(builder, parent_namespaces);
        self.render_attributes(builder, print_options);

        if !self.is_empty_or_single_empty_text_element() {
            if print_options.pretty
                && print_options.single_line_text_elements
                && self.children.len() == 1
            {
                let mut child_output = String::new();
                if self.children[0].render_single_line(&mut child_output, print_options) {
                    builder.push('>');
                    builder.push_str(&child_output);
                    builder.push_str("</");
                    builder.push_str(&self.node_name);
                    builder.push('>');
                    builder.push_str(line_ending);
                    return;
                }
            }

            builder.push('>');
            builder.push_str(line_ending);
            let child_indent = print_options.child_indent(indent);
            let current_namespaces = self.namespaces_with_parents(parent_namespaces);
            for child in &self.children {
                child.render(builder, &child_indent, print_options, &current_namespaces);
            }
            builder.push_str(indent);
            builder.push_str("</");
            builder.push_str(&self.node_name);
            builder.push('>');
            builder.push_str(line_ending);
        } else {
            if print_options.use_self_closing_tags {
                builder.push_str("/>");
            } else {
                builder.push_str("></");
                builder.push_str(&self.node_name);
                builder.push('>');
            }
            builder.push_str(line_ending);
        }
    }

    fn is_empty_or_single_empty_text_element(&self) -> bool {
        self.children.is_empty() || (self.children.len() == 1 && self.children[0].is_empty_text())
    }

    fn render_namespaces(&self, builder: &mut String, parent_namespaces: &[Namespace]) {
        for namespace in &self.namespaces {
            if !parent_namespaces.contains(namespace) {
                builder.push(' ');
                builder.push_str(&namespace.to_string());
            }
        }
    }

    fn render_attributes(&self, builder: &mut String, print_options: &PrintOptions) {
        for (name, value) in &self.attributes {
            let text = if value.is_unsafe() {
                value.rendered_raw().unwrap_or("null").to_owned()
            } else {
                escape_value(
                    value.rendered_raw().unwrap_or(""),
                    print_options.xml_version,
                    print_options.use_character_reference,
                )
            };

            builder.push(' ');
            builder.push_str(name);
            builder.push_str("=\"");
            builder.push_str(&text);
            builder.push('"');
        }
    }

    fn namespaces_with_parents(&self, parent_namespaces: &[Namespace]) -> Vec<Namespace> {
        let mut namespaces = parent_namespaces.to_vec();
        for namespace in &self.namespaces {
            if !namespaces.contains(namespace) {
                namespaces.push(namespace.clone());
            }
        }
        namespaces
    }

    fn find_index(&self, element: &Element) -> Result<usize, String> {
        self.children
            .iter()
            .position(|child| child == element)
            .ok_or_else(|| format!("Element is not a child of '{}'", self.node_name))
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new("")
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string_with_options(PrintOptions::default()))
    }
}
