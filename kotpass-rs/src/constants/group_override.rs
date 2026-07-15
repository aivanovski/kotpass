/// Specifies whether a feature for a group is inherited from its parent
/// or is explicitly enabled/disabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GroupOverride {
    #[default]
    Inherit,
    Enabled,
    Disabled,
}

impl GroupOverride {
    pub const fn from_bool(value: Option<bool>) -> Self {
        match value {
            None => Self::Inherit,
            Some(true) => Self::Enabled,
            Some(false) => Self::Disabled,
        }
    }

    pub const fn xml_value(self) -> &'static str {
        match self {
            Self::Inherit => "Null",
            Self::Enabled => "True",
            Self::Disabled => "False",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GroupOverride;

    #[test]
    fn values_match_kotlin_node_mapping() {
        assert_eq!(GroupOverride::from_bool(None), GroupOverride::Inherit);
        assert_eq!(GroupOverride::from_bool(Some(true)), GroupOverride::Enabled);
        assert_eq!(
            GroupOverride::from_bool(Some(false)),
            GroupOverride::Disabled
        );
    }

    #[test]
    fn xml_values_match_format_xml_values() {
        assert_eq!(GroupOverride::Inherit.xml_value(), "Null");
        assert_eq!(GroupOverride::Enabled.xml_value(), "True");
        assert_eq!(GroupOverride::Disabled.xml_value(), "False");
    }
}
