#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AutoTypeItem {
    pub window: String,
    pub keystroke_sequence: String,
}

impl AutoTypeItem {
    pub fn new(window: impl Into<String>, keystroke_sequence: impl Into<String>) -> Self {
        Self {
            window: window.into(),
            keystroke_sequence: keystroke_sequence.into(),
        }
    }
}
