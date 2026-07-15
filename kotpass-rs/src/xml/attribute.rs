use super::Namespace;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Attribute {
    pub name: String,
    pub value: String,
    pub namespace: Option<Namespace>,
}

impl Attribute {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            namespace: None,
        }
    }

    pub fn namespaced(
        name: impl Into<String>,
        value: impl Into<String>,
        namespace: Namespace,
    ) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            namespace: Some(namespace),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AttributeValue {
    Safe(String),
    Unsafe(UnsafeValue),
}

impl AttributeValue {
    pub fn as_deref(&self) -> Option<&str> {
        match self {
            Self::Safe(value) => Some(value),
            Self::Unsafe(value) => value.value.as_deref(),
        }
    }

    pub(crate) fn rendered_raw(&self) -> Option<&str> {
        match self {
            Self::Safe(value) => Some(value),
            Self::Unsafe(value) => value.value.as_deref(),
        }
    }

    pub(crate) fn is_unsafe(&self) -> bool {
        matches!(self, Self::Unsafe(_))
    }
}

impl From<String> for AttributeValue {
    fn from(value: String) -> Self {
        Self::Safe(value)
    }
}

impl From<&str> for AttributeValue {
    fn from(value: &str) -> Self {
        Self::Safe(value.to_owned())
    }
}

impl From<UnsafeValue> for AttributeValue {
    fn from(value: UnsafeValue) -> Self {
        Self::Unsafe(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnsafeValue {
    pub value: Option<String>,
}

pub fn unsafe_value(value: impl Into<String>) -> UnsafeValue {
    UnsafeValue {
        value: Some(value.into()),
    }
}
