use indexmap::{IndexMap, IndexSet};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::constants::{MemoryProtectionFlag, defaults};

use super::{BinaryData, CustomDataValue, CustomIcon};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Meta {
    pub generator: String,
    pub header_hash: Option<Vec<u8>>,
    pub settings_changed: Option<OffsetDateTime>,
    pub name: String,
    pub name_changed: Option<OffsetDateTime>,
    pub description: String,
    pub description_changed: Option<OffsetDateTime>,
    pub default_user: String,
    pub default_user_changed: Option<OffsetDateTime>,
    pub maintenance_history_days: u32,
    pub color: Option<String>,
    pub master_key_changed: Option<OffsetDateTime>,
    pub master_key_change_rec: i32,
    pub master_key_change_force: i32,
    pub recycle_bin_enabled: bool,
    pub recycle_bin_uuid: Option<Uuid>,
    pub recycle_bin_changed: Option<OffsetDateTime>,
    pub entry_templates_group: Option<Uuid>,
    pub entry_templates_group_changed: Option<OffsetDateTime>,
    pub history_max_items: i32,
    pub history_max_size: i32,
    pub last_selected_group: Option<Uuid>,
    pub last_top_visible_group: Option<Uuid>,
    pub memory_protection: IndexSet<MemoryProtectionFlag>,
    pub custom_icons: IndexMap<Uuid, CustomIcon>,
    pub custom_data: IndexMap<String, CustomDataValue>,
    pub binaries: IndexMap<Vec<u8>, BinaryData>,
}

impl Default for Meta {
    fn default() -> Self {
        let now = OffsetDateTime::now_utc();
        let mut memory_protection = IndexSet::new();
        memory_protection.insert(MemoryProtectionFlag::Password);

        Self {
            generator: defaults::GENERATOR.to_owned(),
            header_hash: None,
            settings_changed: Some(now),
            name: String::new(),
            name_changed: Some(now),
            description: String::new(),
            description_changed: Some(now),
            default_user: String::new(),
            default_user_changed: Some(now),
            maintenance_history_days: defaults::MAINTENANCE_HISTORY_DAYS,
            color: None,
            master_key_changed: None,
            master_key_change_rec: -1,
            master_key_change_force: -1,
            recycle_bin_enabled: false,
            recycle_bin_uuid: None,
            recycle_bin_changed: None,
            entry_templates_group: None,
            entry_templates_group_changed: None,
            history_max_items: defaults::HISTORY_MAX_ITEMS,
            history_max_size: defaults::HISTORY_MAX_SIZE,
            last_selected_group: None,
            last_top_visible_group: None,
            memory_protection,
            custom_icons: IndexMap::new(),
            custom_data: IndexMap::new(),
            binaries: IndexMap::new(),
        }
    }
}
