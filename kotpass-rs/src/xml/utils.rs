use super::{Namespace, XmlVersion};

pub fn escape_value(
    value: impl AsRef<str>,
    _xml_version: XmlVersion,
    use_character_reference: bool,
) -> String {
    let value = value.as_ref();
    if use_character_reference {
        return reference_character(value);
    }

    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\'' => escaped.push_str("&apos;"),
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

pub fn reference_character(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\'' => escaped.push_str("&#39;"),
            '&' => escaped.push_str("&#38;"),
            '<' => escaped.push_str("&#60;"),
            '>' => escaped.push_str("&#62;"),
            '"' => escaped.push_str("&#34;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

pub fn build_name(name: &str, namespace: Option<&Namespace>) -> String {
    if let Some(namespace) = namespace
        && !namespace.is_default()
    {
        return format!("{}:{name}", namespace.name);
    }
    name.to_owned()
}
