use crate::crypto::EncryptedValue;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EntryValue {
    Plain(String),
    Encrypted(EncryptedValue),
}

impl EntryValue {
    pub fn plain(content: impl Into<String>) -> Self {
        Self::Plain(content.into())
    }

    pub fn encrypted(value: EncryptedValue) -> Self {
        Self::Encrypted(value)
    }

    pub fn content(&self) -> String {
        match self {
            Self::Plain(content) => content.clone(),
            Self::Encrypted(value) => value.text(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Plain(content) => content.is_empty(),
            Self::Encrypted(value) => value.byte_len() == 0,
        }
    }

    pub fn map<F>(&self, block: F) -> Result<Self, rand::rngs::SysError>
    where
        F: FnOnce(&str) -> String,
    {
        let mapped = block(&self.content());
        match self {
            Self::Plain(_) => Ok(Self::Plain(mapped)),
            Self::Encrypted(_) => Ok(Self::Encrypted(EncryptedValue::from_string(mapped)?)),
        }
    }
}
