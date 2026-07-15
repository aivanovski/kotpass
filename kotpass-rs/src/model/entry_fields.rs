use indexmap::IndexMap;

use crate::{constants::BasicField, crypto::EncryptedValue};

use super::EntryValue;

#[derive(Debug, Clone, Eq)]
pub struct EntryFields {
    fields: IndexMap<String, EntryValue>,
}

impl EntryFields {
    pub fn new(fields: IndexMap<String, EntryValue>) -> Self {
        Self { fields }
    }

    pub fn of(pairs: impl IntoIterator<Item = (String, EntryValue)>) -> Self {
        Self {
            fields: pairs.into_iter().collect(),
        }
    }

    pub fn create_default() -> Result<Self, rand::rngs::SysError> {
        let mut fields = IndexMap::new();

        for field in BasicField::ALL {
            if field != BasicField::Password {
                fields.insert(field.key().to_owned(), EntryValue::Plain(String::new()));
            }
        }

        fields.insert(
            BasicField::Password.key().to_owned(),
            EntryValue::Encrypted(EncryptedValue::from_string("")?),
        );

        Ok(Self { fields })
    }

    pub fn get(&self, key: &str) -> Option<&EntryValue> {
        self.fields.get(key)
    }

    pub fn get_basic(&self, key: BasicField) -> Option<&EntryValue> {
        self.get(key.key())
    }

    pub fn title(&self) -> Option<&EntryValue> {
        self.get_basic(BasicField::Title)
    }

    pub fn user_name(&self) -> Option<&EntryValue> {
        self.get_basic(BasicField::UserName)
    }

    pub fn password(&self) -> Option<&EntryValue> {
        self.get_basic(BasicField::Password)
    }

    pub fn url(&self) -> Option<&EntryValue> {
        self.get_basic(BasicField::Url)
    }

    pub fn notes(&self) -> Option<&EntryValue> {
        self.get_basic(BasicField::Notes)
    }

    pub fn insert(&mut self, key: impl Into<String>, value: EntryValue) -> Option<EntryValue> {
        self.fields.insert(key.into(), value)
    }

    pub fn remove(&mut self, key: &str) -> Option<EntryValue> {
        self.fields.shift_remove(key)
    }

    pub fn iter(&self) -> indexmap::map::Iter<'_, String, EntryValue> {
        self.fields.iter()
    }

    pub fn inner(&self) -> &IndexMap<String, EntryValue> {
        &self.fields
    }

    pub fn into_inner(self) -> IndexMap<String, EntryValue> {
        self.fields
    }
}

impl PartialEq for EntryFields {
    fn eq(&self, other: &Self) -> bool {
        self.fields.iter().eq(other.fields.iter())
    }
}

impl Default for EntryFields {
    fn default() -> Self {
        Self::create_default().expect("system random should be available")
    }
}

impl IntoIterator for EntryFields {
    type Item = (String, EntryValue);
    type IntoIter = indexmap::map::IntoIter<String, EntryValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.into_iter()
    }
}

impl<'a> IntoIterator for &'a EntryFields {
    type Item = (&'a String, &'a EntryValue);
    type IntoIter = indexmap::map::Iter<'a, String, EntryValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::EntryFields;
    use crate::{constants::BasicField, model::EntryValue};

    #[test]
    fn default_fields_match_kotlin_contract() {
        let fields = EntryFields::create_default().unwrap();

        assert_eq!(fields.title(), Some(&EntryValue::Plain(String::new())));
        assert_eq!(fields.user_name(), Some(&EntryValue::Plain(String::new())));
        assert!(fields.password().unwrap().is_empty());
        assert_eq!(fields.url(), Some(&EntryValue::Plain(String::new())));
        assert_eq!(fields.notes(), Some(&EntryValue::Plain(String::new())));
    }

    #[test]
    fn equality_is_order_sensitive() {
        let left = EntryFields::of([
            ("a".to_owned(), EntryValue::Plain("1".to_owned())),
            ("b".to_owned(), EntryValue::Plain("2".to_owned())),
        ]);
        let right = EntryFields::of([
            ("b".to_owned(), EntryValue::Plain("2".to_owned())),
            ("a".to_owned(), EntryValue::Plain("1".to_owned())),
        ]);

        assert_ne!(left, right);
        assert_eq!(left.get_basic(BasicField::Title), None);
    }
}
