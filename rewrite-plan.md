# Kotpass Rust Rewrite Plan

## Goal

Rewrite Kotpass as a Rust library that preserves KDBX read/write behavior first, then improves API ergonomics where Rust gives better primitives. Treat the existing Kotlin implementation and fixtures as the compatibility oracle.

## Current Project Shape

The Kotlin project has these main areas:

1. KDBX database API: `KeePassDatabase`, immutable models, modifier extensions.
2. Binary format handling: headers, inner headers, variant dictionaries, content blocks.
3. Cryptography: AES, Twofish, Salsa20, ChaCha, Argon2, Blake2b, KDFs, HMAC/SHA.
4. XML model parsing/rendering for KeePass content.
5. Tests and fixtures: `.kdbx` files, XML snapshots, crypto vectors.

## Recommended Rust Crate Layout

```text
kotpass-rs/
  Cargo.toml
  src/
    lib.rs
    error.rs
    database/
      mod.rs
      decode.rs
      encode.rs
      credentials.rs
      content_blocks.rs
      placeholders.rs
      modifiers.rs
      header/
        mod.rs
        signature.rs
        database_header.rs
        inner_header.rs
        variant_dictionary.rs
        kdf_parameters.rs
    model/
      mod.rs
      database_content.rs
      group.rs
      entry.rs
      meta.rs
      binaries.rs
      time.rs
    crypto/
      mod.rs
      key_transform.rs
      protected_value.rs
      cipher.rs
      kdf.rs
      stream.rs
    xml/
      mod.rs
      parse.rs
      write.rs
    io/
      mod.rs
      le.rs
  tests/
    fixtures/
```

## Crates To Consider

Use established Rust crates where possible instead of porting crypto engines manually:

```toml
aes = "0.8"
cbc = "0.1"
cipher = "0.4"
twofish = "0.7"
salsa20 = "0.10"
chacha20 = "0.9"
argon2 = "0.5"
blake2 = "0.10"
sha2 = "0.10"
hmac = "0.12"
flate2 = "1"
quick-xml = "0.36"
uuid = "1"
chrono = "0.4"
thiserror = "1"
zeroize = "1"
base64 = "0.22"
hex = "0.4"
indexmap = "2"
```

## Phased Plan

### 1. Freeze Compatibility

- Keep the Kotlin project intact during the rewrite.
- Copy existing `.kdbx`, XML, and crypto fixtures into Rust tests.
- Add Kotlin-side golden outputs if missing: decoded model snapshots, re-encoded database round trips, XML snapshots.
- Define what compatibility means:
  - Decode existing KDBX v3/v4 files.
  - Encode files readable by Kotpass/KeePass.
  - Preserve protected values, binaries, timestamps, UUIDs, groups, entries, custom data.

### 2. Create Rust Skeleton

- Add a new Rust crate, likely `kotpass-rs`, beside the Kotlin module.
- Start as a library crate, not a CLI.
- Add CI steps for `cargo test`, `cargo fmt --check`, and `cargo clippy`.
- Define top-level API:

```rust
pub struct KeePassDatabase {
    // ...
}

impl KeePassDatabase {
    pub fn decode<R: Read>(reader: R, credentials: Credentials) -> Result<Self>;
    pub fn encode<W: Write>(&self, writer: W) -> Result<()>;
    pub fn encode_xml(&self) -> Result<String>;
}
```

### 3. Port Low-Level Binary Format

- Implement little-endian readers/writers.
- Port:
  - `Signature`
  - `FormatVersion`
  - `DatabaseHeader`
  - `DatabaseInnerHeader`
  - `VariantDictionary`
  - `KdfParameters`
  - `ContentBlocks`
- Test with existing resources:
  - `kdf_params`
  - `inner_header_with_binaries`
  - header specs
  - content block specs

This phase should not touch XML or full database decoding yet.

### 4. Port Credentials And Key Derivation

- Port `Credentials`.
- Use `zeroize` for sensitive byte buffers.
- Implement composite key generation.
- Implement AES-KDF and Argon2-KDF.
- Test against:
  - `CredentialsSpec`
  - `AesKdfSpec`
  - `Argon2Spec`
  - `Blake2bDigestSpec`

### 5. Port Crypto Providers

- Implement cipher abstraction similar to Kotlin `CipherProvider`.
- Support:
  - AES-CBC with PKCS#7
  - Twofish-CBC with PKCS#7
  - Salsa20 protected stream
  - ChaCha20 protected stream
- Prefer RustCrypto crates over manual ports unless a crate fails compatibility.
- Test against existing stream and block cipher vectors.

### 6. Decode Raw KDBX Content

- Implement full binary decrypt path:
  - Read header.
  - Validate signature/version.
  - Transform key.
  - Validate v4 header SHA/HMAC.
  - Decrypt encrypted payload.
  - Inflate GZip if needed.
  - Read v4 inner header.
- At this point, return raw XML bytes plus binary metadata.
- Test with:
  - `ver3_aes.kdbx`
  - `ver4_aes.kdbx`
  - `ver4_argon2.kdbx`
  - `ver4_twofish.kdbx`
  - invalid/corrupt fixtures

### 7. Port Data Models

- Translate Kotlin data classes into Rust structs/enums.
- Use owned data first for simplicity.
- Use `IndexMap` where XML ordering matters.
- Model protected values explicitly:

```rust
pub enum EntryValue {
    Plain(String),
    Protected(ProtectedValue),
    BinaryRef(BinaryReference),
}
```

- Avoid over-optimizing lifetimes early; compatibility matters more.

### 8. Port XML Parser/Writer

- Use `quick-xml`.
- Implement KeePass XML parsing directly rather than porting the embedded Kotlin XML builder.
- Preserve:
  - protected fields
  - binaries
  - custom icons
  - custom data
  - deleted objects
  - auto-type
  - timestamps
- Test with existing XML fixtures:
  - `entry.xml`
  - `group.xml`
  - `meta.xml`
  - `custom_icons.xml`
  - `autotype.xml`

### 9. Implement Full Decode

- Combine binary decrypt plus XML parser.
- Match Kotlin behavior for:
  - unsupported versions
  - wrong credentials
  - invalid header hash
  - missing fields
  - untitled labels
- Test against all `.kdbx` resources.

### 10. Implement Full Encode

- Marshal models to XML.
- Write inner header for v4.
- Compress if needed.
- Encrypt.
- Write content blocks.
- Write header hash/HMAC.
- Round-trip tests:
  - Decode Kotlin fixture in Rust.
  - Encode in Rust.
  - Decode Rust output in Rust.
  - Ideally decode Rust output using existing Kotlin tests too.

### 11. Port Modifiers And Builders

- Kotlin modifier extensions should become Rust methods or builder-style APIs:

```rust
db.modify_meta(|meta| {
    meta.generator = Some("kotpass-rs".into());
});

db.find_entry(|entry| entry.title() == Some("GitHub"));
```

- Decide whether Rust API should be immutable like Kotlin or mutable with explicit cloning.
- Start mutable internally, then expose ergonomic immutable helpers if needed.

### 12. Interop Validation

- Add a compatibility test harness:
  - Kotlin decodes files written by Rust.
  - Rust decodes files written by Kotlin.
- This can be a Gradle/Cargo integration script later.
- Keep known-good sample databases under `tests/fixtures`.

### 13. Publication

- Decide crate name, probably `kotpass` if available or `kotpass-rs`.
- Add docs with examples equivalent to current README.
- Add feature flags if needed:
  - `argon2`
  - `twofish`
  - `serde`
  - `chrono`
- Publish only after decode/encode compatibility is stable.

## Suggested Milestone Order

1. Rust crate skeleton and copied fixtures.
2. Header, variant dictionary, content block tests passing.
3. KDF and crypto vector tests passing.
4. Raw KDBX decrypt to XML bytes.
5. XML parse into Rust models.
6. Full decode for v3/v4 fixtures.
7. Full encode and round-trip tests.
8. Public API polish.
9. Kotlin/Rust cross-compatibility harness.
10. Release.

## Main Risks

The riskiest parts are byte-for-byte format compatibility, protected stream encryption, timestamp handling, XML ordering, and v4 HMAC/content block semantics. Those should get tests before broad API work.

Start by adding `kotpass-rs/` as a sibling crate and porting `Signature`, `FormatVersion`, `VariantDictionary`, `KdfParameters`, and `DatabaseHeader` first. That creates a useful foundation without touching the hardest XML/model layer yet.
