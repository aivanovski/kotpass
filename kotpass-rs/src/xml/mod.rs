mod attribute;
mod auto_type_data;
mod boolean;
mod doctype;
mod element;
pub mod format_xml;
mod instant;
pub mod keyfile_xml;
mod namespace;
mod node;
mod node_ext;
mod print_options;
mod text;
mod utils;
mod xml_builder;
mod xml_version;

pub use attribute::{Attribute, AttributeValue, UnsafeValue, unsafe_value};
pub use auto_type_data::{marshal_auto_type_data, unmarshal_auto_type_data};
pub use boolean::{BoolXmlExt, bool_to_xml_string};
pub use doctype::Doctype;
pub use element::Element;
pub use instant::{
    EPOCH_SECONDS_FROM_AD, NodeInstantExt, XmlInstantError, marshal_instant, parse_instant,
};
pub use namespace::Namespace;
pub use node::Node;
pub use node_ext::{NodeValueError, NodeXmlExt};
pub use print_options::PrintOptions;
pub use text::{CDataElement, Comment, ProcessingInstructionElement, TextElement};
pub use utils::{build_name, escape_value, reference_character};
pub use xml_builder::{XmlParseError, node, parse_str, xml};
pub use xml_version::XmlVersion;
