use super::XmlVersion;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintOptions {
    pub pretty: bool,
    pub single_line_text_elements: bool,
    pub use_self_closing_tags: bool,
    pub use_character_reference: bool,
    pub indent: String,
    pub(crate) xml_version: XmlVersion,
}

impl PrintOptions {
    pub fn compact() -> Self {
        Self {
            pretty: false,
            ..Self::default()
        }
    }

    pub(crate) fn line_ending(&self) -> &'static str {
        if self.pretty { "\n" } else { "" }
    }

    pub(crate) fn child_indent(&self, indent: &str) -> String {
        if self.pretty {
            format!("{indent}{}", self.indent)
        } else {
            String::new()
        }
    }
}

impl Default for PrintOptions {
    fn default() -> Self {
        Self {
            pretty: true,
            single_line_text_elements: false,
            use_self_closing_tags: true,
            use_character_reference: false,
            indent: "\t".to_owned(),
            xml_version: XmlVersion::V10,
        }
    }
}
