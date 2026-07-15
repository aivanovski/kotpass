use super::{
    CDataElement, Comment, Namespace, Node, PrintOptions, ProcessingInstructionElement, TextElement,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Element {
    Node(Box<Node>),
    Text(TextElement),
    CData(CDataElement),
    Comment(Comment),
    ProcessingInstruction(ProcessingInstructionElement),
}

impl Element {
    pub(crate) fn render(
        &self,
        builder: &mut String,
        indent: &str,
        print_options: &PrintOptions,
        parent_namespaces: &[Namespace],
    ) {
        match self {
            Self::Node(node) => node.render(builder, indent, print_options, parent_namespaces),
            Self::Text(text) => text.render(builder, indent, print_options),
            Self::CData(cdata) => cdata.render(builder, indent, print_options),
            Self::Comment(comment) => comment.render(builder, indent, print_options),
            Self::ProcessingInstruction(instruction) => {
                instruction.render(builder, indent, print_options);
            }
        }
    }

    pub(crate) fn render_single_line(
        &self,
        builder: &mut String,
        print_options: &PrintOptions,
    ) -> bool {
        match self {
            Self::Text(text) => {
                text.render_single_line(builder, print_options);
                true
            }
            Self::CData(cdata) => {
                cdata.render_single_line(builder);
                true
            }
            Self::ProcessingInstruction(instruction) => {
                instruction.render_single_line(builder);
                true
            }
            Self::Node(_) | Self::Comment(_) => false,
        }
    }

    pub(crate) fn is_empty_text(&self) -> bool {
        matches!(self, Self::Text(text) if text.text.is_empty())
    }

    pub fn as_node(&self) -> Option<&Node> {
        match self {
            Self::Node(node) => Some(node),
            _ => None,
        }
    }
}

impl From<Node> for Element {
    fn from(value: Node) -> Self {
        Self::Node(Box::new(value))
    }
}

impl From<TextElement> for Element {
    fn from(value: TextElement) -> Self {
        Self::Text(value)
    }
}

impl From<CDataElement> for Element {
    fn from(value: CDataElement) -> Self {
        Self::CData(value)
    }
}

impl From<Comment> for Element {
    fn from(value: Comment) -> Self {
        Self::Comment(value)
    }
}

impl From<ProcessingInstructionElement> for Element {
    fn from(value: ProcessingInstructionElement) -> Self {
        Self::ProcessingInstruction(value)
    }
}
