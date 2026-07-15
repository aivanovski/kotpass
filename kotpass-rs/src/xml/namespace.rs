use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Namespace {
    pub name: String,
    pub value: String,
}

impl Namespace {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }

    pub fn default(value: impl Into<String>) -> Self {
        Self::new("", value)
    }

    pub fn is_default(&self) -> bool {
        self.name.is_empty()
    }

    pub fn fq_name(&self) -> String {
        if self.is_default() {
            "xmlns".to_owned()
        } else {
            format!("xmlns:{}", self.name)
        }
    }
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}=\"{}\"", self.fq_name(), self.value)
    }
}
