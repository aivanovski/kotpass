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

- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/constants/AutoTypeObfuscation.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/constants/BasicField.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/constants/Const.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/constants/CrsAlgorithm.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/constants/Defaults.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/constants/FieldReference.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/constants/GroupOverride.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/constants/HeaderFieldId.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/constants/KdfConst.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/constants/Placeholder.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/constants/PredefinedIcon.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/constants/VariantTypeId.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/constants/MemoryProtectionFlag.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/errors/CryptoError.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/errors/FormatError.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/errors/KeyfileError.kt`

## Phase 2: low-level byte, IO, and stream helpers

- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/ByteUtils.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/extensions/ByteArray.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/extensions/ByteString.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/extensions/Long.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/extensions/SecureRandom.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/io/Base16.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/io/Base64.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/io/BufferedStream.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/io/RealBufferedStream.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/io/TeeBufferedStream.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/extensions/Source.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/extensions/UUID.kt`

## Phase 3: core data models

- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/AutoTypeItem.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/BinaryReference.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/CustomDataValue.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/CustomIcon.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/DatabaseContent.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/DeletedObject.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/TimeData.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/AutoTypeData.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/BinaryData.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/DatabaseElement.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/EntryValue.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/EntryFields.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/Entry.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/FormatVersion.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/Group.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/Meta.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/models/XmlContext.kt`

## Phase 4: vendored XML builder package

- [ ] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/Attribute.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/CDATAElement.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/Comment.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/Doctype.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/Element.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/Namespace.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/Node.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/PrintOptions.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/ProcessingInstructionElement.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/TextElement.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/Unsafe.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/Utils.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/XmlBuilder.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/org/redundent/kotlin/xml/XmlVersion.kt`

## Phase 5: crypto primitives and providers

- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/EncryptedValue.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/block/BlockCipher.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/block/BlockCipherMode.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/block/PaddedBufferedBlockCipher.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/padding/BlockCipherPadding.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/padding/PKCS7Padding.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/engines/ChaChaCore.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/engines/ChaCha7539Engine.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/engines/ChaChaEngine.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/engines/Salsa20Engine.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/engines/TwofishEngine.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/engines/Blake2bDigest.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/engines/Argon2Engine.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/EncryptionSaltGenerator.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/KeyTransform.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/format/CipherProvider.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/format/AesKdf.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/format/Argon2Kdf.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/format/BaseCiphers.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/format/TwofishCipher.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/format/BaseKdfProvider.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/cryptography/format/KdfProvider.kt`

## Phase 6: XML mapping for KeePass models

- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/xml/FormatXml.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/xml/KeyfileXml.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/extensions/Boolean.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/extensions/Node.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/xml/Instant.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/xml/AutoTypeData.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/xml/Binaries.kt`
- [ ] `kotpass/kotpass/src/main/kotlin/app/keemobile/kotpass/xml/BinaryReference.kt`
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
