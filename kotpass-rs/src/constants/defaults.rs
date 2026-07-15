pub const GENERATOR: &str = "Kotpass";
pub const PLACEHOLDERS_MAX_DEPTH: u32 = 10;
pub const MAINTENANCE_HISTORY_DAYS: u32 = 365;
pub const HISTORY_MAX_ITEMS: i32 = 10;
pub const HISTORY_MAX_SIZE: i32 = 6 * 1024 * 1024;
pub const RECYCLE_BIN_NAME: &str = "Recycle Bin";
pub const UNTITLED_LABEL: &str = "Untitled";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_match_kotlin_defaults() {
        assert_eq!(GENERATOR, "Kotpass");
        assert_eq!(PLACEHOLDERS_MAX_DEPTH, 10);
        assert_eq!(MAINTENANCE_HISTORY_DAYS, 365);
        assert_eq!(HISTORY_MAX_ITEMS, 10);
        assert_eq!(HISTORY_MAX_SIZE, 6 * 1024 * 1024);
        assert_eq!(RECYCLE_BIN_NAME, "Recycle Bin");
        assert_eq!(UNTITLED_LABEL, "Untitled");
    }
}
