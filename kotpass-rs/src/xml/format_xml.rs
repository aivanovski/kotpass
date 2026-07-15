pub mod tags {
    pub const DOCUMENT: &str = "KeePassFile";
    pub const ROOT: &str = "Root";
    pub const UUID: &str = "UUID";

    pub mod meta {
        use super::UUID;

        pub const TAG_NAME: &str = "Meta";
        pub const GENERATOR: &str = "Generator";
        pub const HEADER_HASH: &str = "HeaderHash";
        pub const SETTINGS_CHANGED: &str = "SettingsChanged";
        pub const DATABASE_NAME: &str = "DatabaseName";
        pub const DATABASE_NAME_CHANGED: &str = "DatabaseNameChanged";
        pub const DATABASE_DESCRIPTION: &str = "DatabaseDescription";
        pub const DATABASE_DESCRIPTION_CHANGED: &str = "DatabaseDescriptionChanged";
        pub const DEFAULT_USER_NAME: &str = "DefaultUserName";
        pub const DEFAULT_USER_NAME_CHANGED: &str = "DefaultUserNameChanged";
        pub const MAINTENANCE_HISTORY_DAYS: &str = "MaintenanceHistoryDays";
        pub const COLOR: &str = "Color";
        pub const MASTER_KEY_CHANGED: &str = "MasterKeyChanged";
        pub const MASTER_KEY_CHANGE_REC: &str = "MasterKeyChangeRec";
        pub const MASTER_KEY_CHANGE_FORCE: &str = "MasterKeyChangeForce";
        pub const RECYCLE_BIN_ENABLED: &str = "RecycleBinEnabled";
        pub const RECYCLE_BIN_UUID: &str = "RecycleBinUUID";
        pub const RECYCLE_BIN_CHANGED: &str = "RecycleBinChanged";
        pub const ENTRY_TEMPLATES_GROUP: &str = "EntryTemplatesGroup";
        pub const ENTRY_TEMPLATES_GROUP_CHANGED: &str = "EntryTemplatesGroupChanged";
        pub const HISTORY_MAX_ITEMS: &str = "HistoryMaxItems";
        pub const HISTORY_MAX_SIZE: &str = "HistoryMaxSize";
        pub const LAST_SELECTED_GROUP: &str = "LastSelectedGroup";
        pub const LAST_TOP_VISIBLE_GROUP: &str = "LastTopVisibleGroup";

        pub mod binaries {
            pub const TAG_NAME: &str = "Binaries";
            pub const ITEM: &str = "Binary";
        }

        pub mod memory_protection {
            pub const TAG_NAME: &str = "MemoryProtection";
            pub const PROTECT_TITLE: &str = "ProtectTitle";
            pub const PROTECT_USER_NAME: &str = "ProtectUserName";
            pub const PROTECT_PASSWORD: &str = "ProtectPassword";
            pub const PROTECT_URL: &str = "ProtectURL";
            pub const PROTECT_NOTES: &str = "ProtectNotes";
        }

        pub mod custom_icons {
            use super::UUID;

            pub const TAG_NAME: &str = "CustomIcons";
            pub const ITEM: &str = "Icon";
            pub const ITEM_UUID: &str = UUID;
            pub const ITEM_DATA: &str = "Data";
            pub const ITEM_NAME: &str = "Name";
        }
    }

    pub mod group {
        pub const TAG_NAME: &str = "Group";
        pub const NAME: &str = "Name";
        pub const NOTES: &str = "Notes";
        pub const ICON_ID: &str = "IconID";
        pub const CUSTOM_ICON_ID: &str = "CustomIconUUID";
        pub const TAGS: &str = "Tags";
        pub const IS_EXPANDED: &str = "IsExpanded";
        pub const DEFAULT_AUTO_TYPE_SEQUENCE: &str = "DefaultAutoTypeSequence";
        pub const ENABLE_AUTO_TYPE: &str = "EnableAutoType";
        pub const ENABLE_SEARCHING: &str = "EnableSearching";
        pub const LAST_TOP_VISIBLE_ENTRY: &str = "LastTopVisibleEntry";
        pub const PREVIOUS_PARENT_GROUP: &str = "PreviousParentGroup";
    }

    pub mod entry {
        pub const TAG_NAME: &str = "Entry";
        pub const ICON_ID: &str = "IconID";
        pub const CUSTOM_ICON_ID: &str = "CustomIconUUID";
        pub const FOREGROUND_COLOR: &str = "ForegroundColor";
        pub const BACKGROUND_COLOR: &str = "BackgroundColor";
        pub const OVERRIDE_URL: &str = "OverrideURL";
        pub const TAGS: &str = "Tags";
        pub const HISTORY: &str = "History";
        pub const QUALITY_CHECK: &str = "QualityCheck";
        pub const PREVIOUS_PARENT_GROUP: &str = "PreviousParentGroup";

        pub mod fields {
            pub const TAG_NAME: &str = "String";
            pub const ITEM_KEY: &str = "Key";
            pub const ITEM_VALUE: &str = "Value";
        }

        pub mod binary_references {
            pub const TAG_NAME: &str = "Binary";
            pub const ITEM_KEY: &str = "Key";
            pub const ITEM_VALUE: &str = "Value";
        }

        pub mod auto_type {
            pub const TAG_NAME: &str = "AutoType";
            pub const ENABLED: &str = "Enabled";
            pub const OBFUSCATION: &str = "DataTransferObfuscation";
            pub const DEFAULT_SEQUENCE: &str = "DefaultSequence";
            pub const ASSOCIATION: &str = "Association";
            pub const WINDOW: &str = "Window";
            pub const KEYSTROKE_SEQUENCE: &str = "KeystrokeSequence";
        }
    }

    pub mod custom_data {
        pub const TAG_NAME: &str = "CustomData";
        pub const ITEM: &str = "Item";
        pub const ITEM_KEY: &str = "Key";
        pub const ITEM_VALUE: &str = "Value";
    }

    pub mod time_data {
        pub const TAG_NAME: &str = "Times";
        pub const CREATION_TIME: &str = "CreationTime";
        pub const LAST_MODIFICATION_TIME: &str = "LastModificationTime";
        pub const LAST_ACCESS_TIME: &str = "LastAccessTime";
        pub const EXPIRY_TIME: &str = "ExpiryTime";
        pub const EXPIRES: &str = "Expires";
        pub const USAGE_COUNT: &str = "UsageCount";
        pub const LOCATION_CHANGED: &str = "LocationChanged";
    }

    pub mod deleted_objects {
        pub const TAG_NAME: &str = "DeletedObjects";
        pub const OBJECT: &str = "DeletedObject";
        pub const TIME: &str = "DeletionTime";
    }
}

pub mod attributes {
    pub const ID: &str = "ID";
    pub const REF: &str = "Ref";
    pub const PROTECTED: &str = "Protected";
    pub const PROTECTED_IN_MEM_PLAIN_XML: &str = "ProtectInMemory";
    pub const COMPRESSED: &str = "Compressed";
}

pub mod values {
    pub const TRUE: &str = "True";
    pub const FALSE: &str = "False";
    pub const NULL: &str = "Null";
}

#[cfg(test)]
mod tests {
    use super::{attributes, tags, values};

    #[test]
    fn top_level_values_match_kotlin_constants() {
        assert_eq!(tags::DOCUMENT, "KeePassFile");
        assert_eq!(tags::ROOT, "Root");
        assert_eq!(tags::UUID, "UUID");
        assert_eq!(attributes::PROTECTED_IN_MEM_PLAIN_XML, "ProtectInMemory");
        assert_eq!(values::TRUE, "True");
        assert_eq!(values::FALSE, "False");
        assert_eq!(values::NULL, "Null");
    }

    #[test]
    fn nested_values_match_kotlin_constants() {
        assert_eq!(tags::meta::TAG_NAME, "Meta");
        assert_eq!(tags::meta::RECYCLE_BIN_UUID, "RecycleBinUUID");
        assert_eq!(tags::meta::binaries::ITEM, "Binary");
        assert_eq!(tags::meta::custom_icons::ITEM_UUID, tags::UUID);
        assert_eq!(tags::group::CUSTOM_ICON_ID, "CustomIconUUID");
        assert_eq!(tags::entry::fields::TAG_NAME, "String");
        assert_eq!(
            tags::entry::auto_type::OBFUSCATION,
            "DataTransferObfuscation"
        );
        assert_eq!(tags::custom_data::ITEM_VALUE, "Value");
        assert_eq!(tags::time_data::LOCATION_CHANGED, "LocationChanged");
        assert_eq!(tags::deleted_objects::TIME, "DeletionTime");
    }
}
