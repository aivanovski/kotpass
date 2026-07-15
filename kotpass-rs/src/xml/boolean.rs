use crate::xml::format_xml::values;

pub fn bool_to_xml_string(value: bool) -> &'static str {
    if value { values::TRUE } else { values::FALSE }
}

pub trait BoolXmlExt {
    fn to_xml_string(self) -> &'static str;
}

impl BoolXmlExt for bool {
    fn to_xml_string(self) -> &'static str {
        bool_to_xml_string(self)
    }
}

#[cfg(test)]
mod tests {
    use super::{BoolXmlExt, bool_to_xml_string};
    use crate::xml::format_xml::values;

    #[test]
    fn formats_boolean_values_with_keepass_casing() {
        assert_eq!(bool_to_xml_string(true), values::TRUE);
        assert_eq!(bool_to_xml_string(false), values::FALSE);
        assert_eq!(true.to_xml_string(), "True");
        assert_eq!(false.to_xml_string(), "False");
    }
}
