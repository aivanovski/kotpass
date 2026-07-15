# Kotpass Kotlin to Rust migration progress

Scope: production Kotlin sources under `kotpass/kotpass/src/main/kotlin`.

Total production Kotlin files: 120.

Legend:
- `[ ]` not converted
- `[x]` converted
- Add notes after an item when a Kotlin file is intentionally replaced by a Rust crate or merged into another Rust module.

Ordering notes:
- This is a practical conversion order, not a strict compiler dependency graph.
- Start with files that have no local project imports, then move upward through models, crypto, XML, database format, and public modifiers.
- The vendored `org.redundent.kotlin.xml` package is listed separately because Rust may replace it with XML crates instead of translating it file-for-file.
- Tests under `kotpass/kotpass/src/test/kotlin` are not included here yet; convert each test near the matching production module.

## Phase 1: constants, errors, and small primitives

- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/constants/AutoTypeObfuscation.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/constants/BasicField.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/constants/Const.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/constants/CrsAlgorithm.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/constants/Defaults.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/constants/FieldReference.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/constants/GroupOverride.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/constants/HeaderFieldId.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/constants/KdfConst.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/constants/Placeholder.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/constants/PredefinedIcon.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/constants/VariantTypeId.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/constants/MemoryProtectionFlag.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/errors/CryptoError.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/errors/FormatError.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/errors/KeyfileError.kt`

## Phase 2: low-level byte, IO, and stream helpers

- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/ByteUtils.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/extensions/ByteArray.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/extensions/ByteString.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/extensions/Long.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/extensions/SecureRandom.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/io/Base16.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/io/Base64.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/io/BufferedStream.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/io/RealBufferedStream.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/io/TeeBufferedStream.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/extensions/Source.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/extensions/UUID.kt`

## Phase 3: core data models

- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/AutoTypeItem.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/BinaryReference.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/CustomDataValue.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/CustomIcon.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/DatabaseContent.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/DeletedObject.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/TimeData.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/AutoTypeData.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/BinaryData.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/DatabaseElement.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/EntryValue.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/EntryFields.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/Entry.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/FormatVersion.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/Group.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/Meta.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/XmlContext.kt`

## Phase 4: vendored XML builder package

- [x] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/Attribute.kt` - replaced by `kotpass-rs/src/xml`.
- [x] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/CDATAElement.kt` - replaced by `kotpass-rs/src/xml`.
- [x] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/Comment.kt` - replaced by `kotpass-rs/src/xml`.
- [x] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/Doctype.kt` - replaced by `kotpass-rs/src/xml`.
- [x] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/Element.kt` - replaced by `kotpass-rs/src/xml`.
- [x] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/Namespace.kt` - replaced by `kotpass-rs/src/xml`.
- [x] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/Node.kt` - replaced by `kotpass-rs/src/xml`.
- [x] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/PrintOptions.kt` - replaced by `kotpass-rs/src/xml`.
- [x] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/ProcessingInstructionElement.kt` - replaced by `kotpass-rs/src/xml`.
- [x] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/TextElement.kt` - replaced by `kotpass-rs/src/xml`.
- [x] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/Unsafe.kt` - replaced by `kotpass-rs/src/xml`.
- [x] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/Utils.kt` - replaced by `kotpass-rs/src/xml`.
- [x] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/XmlBuilder.kt` - replaced by `kotpass-rs/src/xml`.
- [x] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/XmlVersion.kt` - replaced by `kotpass-rs/src/xml`.

## Phase 5: crypto primitives and providers

- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/EncryptedValue.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/block/BlockCipher.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/block/BlockCipherMode.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/block/PaddedBufferedBlockCipher.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/padding/BlockCipherPadding.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/padding/PKCS7Padding.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/engines/ChaChaCore.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/engines/ChaCha7539Engine.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/engines/ChaChaEngine.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/engines/Salsa20Engine.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/engines/TwofishEngine.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/engines/Blake2bDigest.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/engines/Argon2Engine.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/EncryptionSaltGenerator.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/KeyTransform.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/format/CipherProvider.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/format/AesKdf.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/format/Argon2Kdf.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/format/BaseCiphers.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/format/TwofishCipher.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/format/BaseKdfProvider.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/format/KdfProvider.kt`

## Phase 6: XML mapping for KeePass models

- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/xml/FormatXml.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/xml/KeyfileXml.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/extensions/Boolean.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/extensions/Node.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/xml/Instant.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/xml/AutoTypeData.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/xml/Binaries.kt`
- [x] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/xml/BinaryReference.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/xml/CustomData.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/xml/CustomIcons.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/xml/DeletedObject.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/xml/TimeData.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/xml/Entry.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/xml/Group.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/xml/Meta.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/xml/XmlContentParser.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/xml/DefaultXmlContentParser.kt`

## Phase 7: database header and content format

- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/database/header/Signature.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/database/header/VariantItem.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/database/header/VariantDictionary.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/database/header/KdfParameters.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/database/header/DatabaseInnerHeader.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/database/header/DatabaseHeader.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/database/ContentBlocks.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/database/Credentials.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/database/Placeholders.kt`

## Phase 8: database API, builders, and modifiers

- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/database/KeePassDatabase.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/database/Decoder.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/database/Encoder.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/builders/Entry.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/builders/Group.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/database/modifiers/Binaries.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/database/modifiers/Content.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/database/modifiers/Credentials.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/database/modifiers/CustomIcons.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/database/modifiers/DatabaseHeader.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/database/modifiers/Entry.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/database/modifiers/Group.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/database/modifiers/Meta.kt`
