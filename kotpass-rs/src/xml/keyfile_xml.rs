pub mod tags {
    pub const DOCUMENT: &str = "KeyFile";
    pub const META: &str = "Meta";
    pub const VERSION: &str = "Version";
    pub const KEY: &str = "Key";
    pub const DATA: &str = "Data";
}

pub mod attributes {
    pub const HASH: &str = "Hash";
}

#[cfg(test)]
mod tests {
    use super::{attributes, tags};

    #[test]
    fn values_match_kotlin_constants() {
        assert_eq!(tags::DOCUMENT, "KeyFile");
        assert_eq!(tags::META, "Meta");
        assert_eq!(tags::VERSION, "Version");
        assert_eq!(tags::KEY, "Key");
        assert_eq!(tags::DATA, "Data");
        assert_eq!(attributes::HASH, "Hash");
    }
}
