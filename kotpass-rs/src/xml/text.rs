use super::{PrintOptions, escape_value};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextElement {
    pub text: String,
    pub unsafe_text: bool,
}

impl TextElement {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            unsafe_text: false,
        }
    }

    pub fn unsafe_text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            unsafe_text: true,
        }
    }

    pub(crate) fn render(&self, builder: &mut String, indent: &str, print_options: &PrintOptions) {
        if self.text.is_empty() {
            return;
        }

        builder.push_str(indent);
        self.render_single_line(builder, print_options);
        builder.push_str(print_options.line_ending());
    }

    pub(crate) fn render_single_line(&self, builder: &mut String, print_options: &PrintOptions) {
        if self.unsafe_text {
            builder.push_str(&self.text);
        } else {
            builder.push_str(&escape_value(
                &self.text,
                print_options.xml_version,
                print_options.use_character_reference,
            ));
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CDataElement {
    pub text: String,
}

impl CDataElement {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    pub(crate) fn render(&self, builder: &mut String, indent: &str, print_options: &PrintOptions) {
        builder.push_str(indent);
        self.render_single_line(builder);
        builder.push_str(print_options.line_ending());
    }

    pub(crate) fn render_single_line(&self, builder: &mut String) {
        builder.push_str("<![CDATA[");
        builder.push_str(&self.text.replace("]]>", "]]]]><![CDATA[>"));
        builder.push_str("]]>");
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Comment {
    pub text: String,
}

impl Comment {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    pub(crate) fn render(&self, builder: &mut String, indent: &str, print_options: &PrintOptions) {
        builder.push_str(indent);
        builder.push_str("<!-- ");
        builder.push_str(&self.text.replace("--", "&#45;&#45;"));
        builder.push_str(" -->");
        builder.push_str(print_options.line_ending());
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProcessingInstructionElement {
    pub text: String,
    pub attributes: Vec<(String, String)>,
}

impl ProcessingInstructionElement {
    pub fn new(text: impl Into<String>, attributes: Vec<(String, String)>) -> Self {
        Self {
            text: text.into(),
            attributes,
        }
    }

    pub(crate) fn render(&self, builder: &mut String, indent: &str, print_options: &PrintOptions) {
        builder.push_str(indent);
        self.render_single_line(builder);
        builder.push_str(print_options.line_ending());
    }

    pub(crate) fn render_single_line(&self, builder: &mut String) {
        builder.push_str("<?");
        builder.push_str(&self.text);
        if !self.attributes.is_empty() {
            builder.push(' ');
            for (index, (key, value)) in self.attributes.iter().enumerate() {
                if index > 0 {
                    builder.push(' ');
                }
                builder.push_str(key);
                builder.push_str("=\"");
                builder.push_str(value);
                builder.push('"');
            }
        }
        builder.push_str("?>");
    }
}
