use regex::Regex;
use std::sync::LazyLock;

pub const TAGS_SEPARATOR: &str = ";";

pub static TAGS_SEPARATORS_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s*[;,:]\s*").expect("tag separator regex is valid"));

pub fn split_tags(tags: &str) -> Vec<&str> {
    TAGS_SEPARATORS_REGEX.split(tags).collect()
}

pub fn bytes<I, T>(values: I) -> Vec<u8>
where
    I: IntoIterator<Item = T>,
    T: Into<i64>,
{
    values.into_iter().map(|value| value.into() as u8).collect()
}

#[cfg(test)]
mod tests {
    use super::{TAGS_SEPARATOR, bytes, split_tags};

    #[test]
    fn tag_separator_matches_kotlin_value() {
        assert_eq!(TAGS_SEPARATOR, ";");
    }

    #[test]
    fn splits_tags_with_kotlin_separator_regex() {
        assert_eq!(
            split_tags("alpha; beta, gamma :delta"),
            vec!["alpha", "beta", "gamma", "delta"]
        );
    }

    #[test]
    fn preserves_outer_whitespace_like_regex_split() {
        assert_eq!(split_tags(" alpha; beta "), vec![" alpha", "beta "]);
    }

    #[test]
    fn bytes_truncates_values_like_kotlin_to_byte() {
        assert_eq!(
            bytes([0x03, 0xD9, 0xA2, 0x9A, 0x100, -1]),
            vec![0x03, 0xD9, 0xA2, 0x9A, 0x00, 0xFF]
        );
    }
}
