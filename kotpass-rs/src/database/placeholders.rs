use std::sync::OnceLock;

use indexmap::IndexMap;
use regex::Regex;
use uuid::Uuid;

use crate::{
    constants::{BasicField, Placeholder, SearchIn, WantedField, defaults::PLACEHOLDERS_MAX_DEPTH},
    io::base16::decode_hex_to_array,
    model::{Entry, EntryValue, Group},
    uuid_ext::to_hex_string,
};

static PLACEHOLDER_REGEX: OnceLock<Regex> = OnceLock::new();
static REFERENCE_REGEX: OnceLock<Regex> = OnceLock::new();

pub struct PlaceholderResolver<'a> {
    entries: Vec<&'a Entry>,
}

impl<'a> PlaceholderResolver<'a> {
    pub fn from_entries(entries: impl IntoIterator<Item = &'a Entry>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
        }
    }

    pub fn from_group(group: &'a Group) -> Self {
        let mut entries = Vec::new();
        collect_group_entries(group, &mut entries);
        Self { entries }
    }

    pub fn resolve_entry_placeholders(
        &self,
        entry: &Entry,
        max_depth: u32,
    ) -> Result<IndexMap<String, EntryValue>, rand::rngs::SysError> {
        let mut result = IndexMap::new();

        for (key, value) in &entry.fields {
            result.insert(
                key.clone(),
                self.resolve_value_placeholders(entry, value, max_depth)?,
            );
        }

        Ok(result)
    }

    pub fn resolve_entry_placeholders_default(
        &self,
        entry: &Entry,
    ) -> Result<IndexMap<String, EntryValue>, rand::rngs::SysError> {
        self.resolve_entry_placeholders(entry, PLACEHOLDERS_MAX_DEPTH)
    }

    pub fn resolve_value_placeholders(
        &self,
        entry: &Entry,
        value: &EntryValue,
        max_depth: u32,
    ) -> Result<EntryValue, rand::rngs::SysError> {
        let content = value.content();
        if !content.contains('{') {
            return Ok(value.clone());
        }

        value.map(|content| {
            placeholder_regex()
                .replace_all(content, |captures: &regex::Captures<'_>| {
                    self.resolve_placeholder(entry, &captures[0], max_depth)
                })
                .into_owned()
        })
    }

    pub fn resolve_placeholder(&self, entry: &Entry, placeholder: &str, max_depth: u32) -> String {
        if max_depth == 0 {
            return placeholder.to_owned();
        }

        let pattern = placeholder.trim_matches(|char| char == '{' || char == '}');
        if starts_with_ignore_ascii_case(pattern, Placeholder::Reference.value()) {
            return self.resolve_reference(placeholder, max_depth);
        }
        if pattern.eq_ignore_ascii_case(Placeholder::Uuid.value()) {
            return to_hex_string(entry.uuid);
        }

        let field_key = if starts_with_ignore_ascii_case(pattern, Placeholder::CustomField.value())
        {
            pattern[Placeholder::CustomField.value().len()..].to_owned()
        } else if pattern.eq_ignore_ascii_case(Placeholder::Title.value()) {
            BasicField::Title.key().to_owned()
        } else if pattern.eq_ignore_ascii_case(Placeholder::UserName.value()) {
            BasicField::UserName.key().to_owned()
        } else if pattern.eq_ignore_ascii_case(Placeholder::Password.value()) {
            BasicField::Password.key().to_owned()
        } else if pattern.eq_ignore_ascii_case(Placeholder::Url.value()) {
            BasicField::Url.key().to_owned()
        } else if pattern.eq_ignore_ascii_case(Placeholder::Notes.value()) {
            BasicField::Notes.key().to_owned()
        } else {
            return placeholder.to_owned();
        };

        let content = entry
            .fields
            .get(&field_key)
            .map(EntryValue::content)
            .unwrap_or_default();

        if !content.contains('{') {
            return content;
        }

        placeholder_regex()
            .replace_all(&content, |captures: &regex::Captures<'_>| {
                self.resolve_placeholder(entry, &captures[0], max_depth - 1)
            })
            .into_owned()
    }

    pub fn resolve_reference(&self, reference: &str, max_depth: u32) -> String {
        let Some(captures) = reference_regex().captures(reference) else {
            return reference.to_owned();
        };
        let wanted_key = captures.get(1).map(|value| value.as_str()).unwrap_or("");
        let search_in_key = captures.get(2).map(|value| value.as_str()).unwrap_or("");
        let search_text = captures.get(3).map(|value| value.as_str()).unwrap_or("");

        let Some(wanted) = WantedField::from_key(wanted_key) else {
            return reference.to_owned();
        };
        let Some(search_in) = SearchIn::from_key(search_in_key) else {
            return reference.to_owned();
        };
        let Some(found_entry) = self.find_entry(search_in, search_text) else {
            return reference.to_owned();
        };
        let Some(raw_text) = wanted_text(found_entry, wanted) else {
            return reference.to_owned();
        };

        if !raw_text.contains('{') {
            return raw_text;
        }

        placeholder_regex()
            .replace_all(&raw_text, |captures: &regex::Captures<'_>| {
                self.resolve_placeholder(found_entry, &captures[0], max_depth.saturating_sub(1))
            })
            .into_owned()
    }

    fn find_entry(&self, search_in: SearchIn, search_text: &str) -> Option<&'a Entry> {
        match search_in {
            SearchIn::Uuid => decode_uuid(search_text).ok().and_then(|uuid| {
                self.entries
                    .iter()
                    .copied()
                    .find(|entry| entry.uuid == uuid)
            }),
            SearchIn::Title => self.find_entry_by_field(BasicField::Title.key(), search_text),
            SearchIn::UserName => self.find_entry_by_field(BasicField::UserName.key(), search_text),
            SearchIn::Password => self.find_entry_by_field(BasicField::Password.key(), search_text),
            SearchIn::Url => self.find_entry_by_field(BasicField::Url.key(), search_text),
            SearchIn::Notes => self.find_entry_by_field(BasicField::Notes.key(), search_text),
            SearchIn::Other => self.entries.iter().copied().find(|entry| {
                entry
                    .fields
                    .iter()
                    .filter(|(key, _)| !BasicField::is_basic_key(key))
                    .any(|(key, _)| find_in_field(entry, key, search_text))
            }),
        }
    }

    fn find_entry_by_field(&self, field_key: &str, search_text: &str) -> Option<&'a Entry> {
        self.entries
            .iter()
            .copied()
            .find(|entry| find_in_field(entry, field_key, search_text))
    }
}

impl<'a> Default for PlaceholderResolver<'a> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

fn collect_group_entries<'a>(group: &'a Group, entries: &mut Vec<&'a Entry>) {
    entries.extend(&group.entries);
    for child in &group.groups {
        collect_group_entries(child, entries);
    }
}

fn wanted_text(entry: &Entry, wanted: WantedField) -> Option<String> {
    match wanted {
        WantedField::Title => field_content(entry, BasicField::Title.key()),
        WantedField::UserName => field_content(entry, BasicField::UserName.key()),
        WantedField::Password => field_content(entry, BasicField::Password.key()),
        WantedField::Url => field_content(entry, BasicField::Url.key()),
        WantedField::Notes => field_content(entry, BasicField::Notes.key()),
        WantedField::Uuid => Some(to_hex_string(entry.uuid)),
    }
}

fn field_content(entry: &Entry, field_key: &str) -> Option<String> {
    entry.fields.get(field_key).map(EntryValue::content)
}

fn find_in_field(entry: &Entry, field_key: &str, text: &str) -> bool {
    entry
        .fields
        .get(field_key)
        .map(|value| value.content().contains(text))
        .unwrap_or(false)
}

fn decode_uuid(hex: &str) -> Result<Uuid, ()> {
    let bytes = decode_hex_to_array(hex).map_err(|_| ())?;
    let bytes: [u8; 16] = bytes.try_into().map_err(|_| ())?;
    Ok(Uuid::from_bytes(bytes))
}

fn starts_with_ignore_ascii_case(value: &str, prefix: &str) -> bool {
    value
        .get(..prefix.len())
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(prefix))
}

fn placeholder_regex() -> &'static Regex {
    PLACEHOLDER_REGEX
        .get_or_init(|| Regex::new(r"\{([ \t\d\p{L}:@]+)\}").expect("valid placeholder regex"))
}

fn reference_regex() -> &'static Regex {
    REFERENCE_REGEX.get_or_init(|| {
        Regex::new(r"(?i)\{REF:([TUPANI])@([TUPANIO]):([ \t\d\p{L}]+)\}")
            .expect("valid reference regex")
    })
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use uuid::Uuid;

    use super::PlaceholderResolver;
    use crate::{
        constants::{BasicField, Placeholder},
        model::{Entry, EntryFields, EntryValue, Group},
        uuid_ext::to_hex_string,
    };

    #[test]
    fn resolves_local_placeholders() {
        let uuid = Uuid::new_v4();
        let content1 = "Lorem";
        let content2 = "ipsum";
        let custom_key1 = "Extra";
        let custom_key2 = "Id";
        let entry = entry_with_fields(
            uuid,
            [
                (
                    BasicField::Title.key(),
                    format!("{{{}}}", Placeholder::UserName.value()),
                ),
                (BasicField::UserName.key(), content1.to_owned()),
                (
                    BasicField::Url.key(),
                    format!("{{{}}}", Placeholder::Notes.value()),
                ),
                (BasicField::Notes.key(), content2.to_owned()),
                (
                    BasicField::Password.key(),
                    format!("{{{}{custom_key1}}}", Placeholder::CustomField.value()),
                ),
                (
                    custom_key1,
                    format!(
                        "{{{}}} {{{}}}",
                        Placeholder::Title.value(),
                        Placeholder::Url.value()
                    ),
                ),
                (
                    custom_key2,
                    format!(
                        "{{{}}} {{{}{custom_key1}}}",
                        Placeholder::Uuid.value(),
                        Placeholder::CustomField.value()
                    ),
                ),
            ],
        );
        let resolver = PlaceholderResolver::from_entries([&entry]);
        let result = resolver.resolve_entry_placeholders_default(&entry).unwrap();

        assert_eq!(result[BasicField::Title.key()].content(), content1);
        assert_eq!(
            result[BasicField::Password.key()].content(),
            format!("{content1} {content2}")
        );
        assert_eq!(
            result[custom_key2].content(),
            format!("{} {content1} {content2}", to_hex_string(uuid))
        );
    }

    #[test]
    fn recursion_is_limited_by_maximum_depth() {
        let max_depth = 3;
        let last = "END";
        let mut fields = Vec::new();
        for index in 0..=10 {
            fields.push((
                index.to_string(),
                format!("{{{}{}}}", Placeholder::CustomField.value(), index + 1),
            ));
        }
        fields.push((max_depth.to_string(), last.to_owned()));
        let fields = fields
            .into_iter()
            .map(|(key, value)| (key, EntryValue::Plain(value)));
        let mut entry = Entry::new(Uuid::new_v4());
        entry.fields = EntryFields::of(fields);
        let resolver = PlaceholderResolver::from_entries([&entry]);
        let result = resolver
            .resolve_entry_placeholders(&entry, max_depth)
            .unwrap();

        assert_eq!(result["0"].content(), last);
    }

    #[test]
    fn resolves_nested_references() {
        let content1 = "Lorem";
        let content2 = "ipsum";
        let uuid1 = Uuid::new_v4();
        let entry1 = entry_with_fields(
            uuid1,
            [
                (BasicField::Title.key(), content1.to_owned()),
                (BasicField::Notes.key(), content2.to_owned()),
            ],
        );
        let uuid2 = Uuid::new_v4();
        let entry2 = entry_with_fields(
            uuid2,
            [(
                BasicField::Url.key(),
                format!(
                    "{{REF:T@I:{}}} {{REF:N@I:{}}}",
                    to_hex_string(uuid1),
                    to_hex_string(uuid1)
                ),
            )],
        );
        let entry_with_refs = entry_with_fields(
            Uuid::new_v4(),
            [(
                BasicField::UserName.key(),
                format!("{{REF:A@I:{}}}", to_hex_string(uuid2)),
            )],
        );
        let resolver = PlaceholderResolver::from_entries([&entry1, &entry2, &entry_with_refs]);
        let result = resolver
            .resolve_entry_placeholders_default(&entry_with_refs)
            .unwrap();

        assert_eq!(
            result[BasicField::UserName.key()].content(),
            format!("{content1} {content2}")
        );
    }

    #[test]
    fn resolves_query_based_references() {
        let user_name = "Допплер";
        let content1 = "Lorem ipsum";
        let content2 = "dolor";
        let uuid1 = Uuid::new_v4();
        let entry1 = entry_with_fields(
            uuid1,
            [
                (BasicField::Title.key(), content1.to_owned()),
                (BasicField::UserName.key(), user_name.to_owned()),
                (BasicField::Notes.key(), content2.to_owned()),
            ],
        );
        let uuid2 = Uuid::new_v4();
        let entry2 = entry_with_fields(
            uuid2,
            [(BasicField::Url.key(), format!("{{REF:N@T:{content1}}}"))],
        );
        let entry_with_refs = entry_with_fields(
            Uuid::new_v4(),
            [
                (
                    BasicField::UserName.key(),
                    format!("{{REF:A@I:{}}}", to_hex_string(uuid2)),
                ),
                (BasicField::Notes.key(), format!("{{REF:N@U:{user_name}}}")),
            ],
        );
        let resolver = PlaceholderResolver::from_entries([&entry1, &entry2, &entry_with_refs]);
        let result = resolver
            .resolve_entry_placeholders_default(&entry_with_refs)
            .unwrap();

        assert_eq!(result[BasicField::UserName.key()].content(), content2);
        assert_eq!(result[BasicField::Notes.key()].content(), content2);
    }

    #[test]
    fn can_collect_entries_from_root_group() {
        let entry = entry_with_fields(
            Uuid::new_v4(),
            [(BasicField::Title.key(), "Title".to_owned())],
        );
        let mut child = Group::new(Uuid::new_v4(), "child");
        child.entries.push(entry);
        let mut root = Group::new(Uuid::new_v4(), "root");
        root.groups.push(child);
        let resolver = PlaceholderResolver::from_group(&root);

        assert_eq!(
            resolver.resolve_reference(
                &format!("{{REF:I@T:{}}}", "Title"),
                crate::constants::defaults::PLACEHOLDERS_MAX_DEPTH
            ),
            to_hex_string(root.groups[0].entries[0].uuid)
        );
    }

    fn entry_with_fields(
        uuid: Uuid,
        fields: impl IntoIterator<Item = (&'static str, String)>,
    ) -> Entry {
        let mut entry = Entry::new(uuid);
        let fields: IndexMap<String, EntryValue> = fields
            .into_iter()
            .map(|(key, value)| (key.to_owned(), EntryValue::Plain(value)))
            .collect();
        entry.fields = EntryFields::new(fields);
        entry
    }
}
