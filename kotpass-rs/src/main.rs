use std::{env, error::Error, fs::File, path::PathBuf};

use kotpass_rs::{
    constants::BasicField,
    crypto::EncryptedValue,
    database::{Credentials, KeePassDatabase},
    model::{Entry, EntryValue, Group},
};

const DEFAULT_DATABASE_PATH: &str = "demo.kdbx";
const DEFAULT_PASSPHRASE: &str = "abc123";

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let database_path = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(default_database_path);
    let passphrase = args.next().unwrap_or_else(|| DEFAULT_PASSPHRASE.to_owned());

    let file = File::open(&database_path)?;
    let passphrase = EncryptedValue::from_string(passphrase)?;
    let credentials = Credentials::from_passphrase(&passphrase)?;
    let database = KeePassDatabase::decode(file, credentials)?;
    let version = database.header().version();

    println!(
        "Database: {} (KDBX {}.{})",
        database_path.display(),
        version.major,
        version.minor
    );
    print_group(&database.content().group, 0);

    Ok(())
}

fn print_group(group: &Group, depth: usize) {
    let indent = "  ".repeat(depth);
    println!("{indent}- Group: {}", display_or_untitled(&group.name));

    for entry in &group.entries {
        print_entry(entry, depth + 1);
    }

    for child in &group.groups {
        print_group(child, depth + 1);
    }
}

fn print_entry(entry: &Entry, depth: usize) {
    let indent = "  ".repeat(depth);
    let title = entry
        .fields
        .title()
        .map(display_value)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "<untitled>".to_owned());

    println!("{indent}- Entry: {title}");

    for field in [
        BasicField::UserName,
        BasicField::Url,
        BasicField::Notes,
        BasicField::Password,
    ] {
        if let Some(value) = entry.fields.get_basic(field) {
            print_field(field.key(), value, depth + 1);
        }
    }

    for (key, value) in &entry.fields {
        if !BasicField::is_basic_key(key) {
            print_field(key, value, depth + 1);
        }
    }
}

fn print_field(key: &str, value: &EntryValue, depth: usize) {
    let content = if is_sensitive_field(key) {
        "[protected]".to_owned()
    } else {
        display_value(value)
    };
    if content.is_empty() {
        return;
    }

    let indent = "  ".repeat(depth);
    println!("{indent}{key}: {content}");
}

fn display_value(value: &EntryValue) -> String {
    match value {
        EntryValue::Plain(content) => content.clone(),
        EntryValue::Encrypted(_) => "[protected]".to_owned(),
    }
}

fn is_sensitive_field(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key == BasicField::Password.key().to_ascii_lowercase()
        || key.contains("otp")
        || key.contains("totp")
}

fn display_or_untitled(value: &str) -> &str {
    if value.is_empty() {
        "<untitled>"
    } else {
        value
    }
}

fn default_database_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_DATABASE_PATH)
}
