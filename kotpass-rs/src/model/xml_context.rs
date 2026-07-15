use indexmap::IndexMap;
use indexmap::IndexSet;

use crate::constants::{BasicField, MemoryProtectionFlag, defaults};

use super::{BinaryData, FormatVersion};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XmlContext<E> {
    Encode(XmlEncodeContext<E>),
    Decode(XmlDecodeContext<E>),
}

impl<E> XmlContext<E> {
    pub fn version(&self) -> FormatVersion {
        match self {
            Self::Encode(context) => context.version(),
            Self::Decode(context) => context.version,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XmlEncodeContext<E> {
    Encrypted {
        version: FormatVersion,
        binaries: IndexMap<Vec<u8>, BinaryData>,
        inner_encryption: E,
    },
    Plain {
        version: FormatVersion,
        binaries: IndexMap<Vec<u8>, BinaryData>,
        memory_protection_flags: IndexSet<MemoryProtectionFlag>,
    },
}

impl<E> XmlEncodeContext<E> {
    pub fn version(&self) -> FormatVersion {
        match self {
            Self::Encrypted { version, .. } | Self::Plain { version, .. } => *version,
        }
    }

    pub fn binaries(&self) -> &IndexMap<Vec<u8>, BinaryData> {
        match self {
            Self::Encrypted { binaries, .. } | Self::Plain { binaries, .. } => binaries,
        }
    }
}

impl<E> XmlEncodeContext<E> {
    pub fn memory_protection_keys(&self) -> IndexSet<&'static str> {
        match self {
            Self::Encrypted { .. } => IndexSet::new(),
            Self::Plain {
                memory_protection_flags,
                ..
            } => memory_protection_flags
                .iter()
                .map(|flag| flag.to_basic_field())
                .map(BasicField::key)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlDecodeContext<E> {
    pub version: FormatVersion,
    pub encryption: E,
    pub binaries: IndexMap<Vec<u8>, BinaryData>,
    pub untitled_label: String,
}

impl<E> XmlDecodeContext<E> {
    pub fn new(
        version: FormatVersion,
        encryption: E,
        binaries: IndexMap<Vec<u8>, BinaryData>,
    ) -> Self {
        Self {
            version,
            encryption,
            binaries,
            untitled_label: defaults::UNTITLED_LABEL.to_owned(),
        }
    }
}
