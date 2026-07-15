use crate::constants::AutoTypeObfuscation;

use super::AutoTypeItem;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AutoTypeData {
    pub enabled: bool,
    pub obfuscation: AutoTypeObfuscation,
    pub default_sequence: Option<String>,
    pub items: Vec<AutoTypeItem>,
}

impl AutoTypeData {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            obfuscation: AutoTypeObfuscation::None,
            default_sequence: None,
            items: Vec::new(),
        }
    }
}
