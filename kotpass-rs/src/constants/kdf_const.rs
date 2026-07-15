pub mod keys {
    pub const UUID: &str = "$UUID";
    pub const ROUNDS: &str = "R";
    pub const SALT_OR_SEED: &str = "S";
    pub const PARALLELISM: &str = "P";
    pub const MEMORY: &str = "M";
    pub const ITERATIONS: &str = "I";
    pub const VERSION: &str = "V";
    pub const SECRET_KEY: &str = "K";
    pub const ASSOC_DATA: &str = "A";
}

#[cfg(test)]
mod tests {
    use super::keys;

    #[test]
    fn keys_match_kotlin_values() {
        assert_eq!(keys::UUID, "$UUID");
        assert_eq!(keys::ROUNDS, "R");
        assert_eq!(keys::SALT_OR_SEED, "S");
        assert_eq!(keys::PARALLELISM, "P");
        assert_eq!(keys::MEMORY, "M");
        assert_eq!(keys::ITERATIONS, "I");
        assert_eq!(keys::VERSION, "V");
        assert_eq!(keys::SECRET_KEY, "K");
        assert_eq!(keys::ASSOC_DATA, "A");
    }
}
