#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BinaryReference {
    pub hash: Vec<u8>,
    pub name: String,
}

impl BinaryReference {
    pub fn new(hash: impl Into<Vec<u8>>, name: impl Into<String>) -> Self {
        Self {
            hash: hash.into(),
            name: name.into(),
        }
    }
}
