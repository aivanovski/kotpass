#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum XmlVersion {
    #[default]
    V10,
    V11,
}

impl XmlVersion {
    pub const fn value(self) -> &'static str {
        match self {
            Self::V10 => "1.0",
            Self::V11 => "1.1",
        }
    }
}
