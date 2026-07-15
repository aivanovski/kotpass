use crate::constants::PredefinedIcon;

use super::{Entry, Group, TimeData};
use uuid::Uuid;

pub trait DatabaseElement {
    fn uuid(&self) -> Uuid;
    fn times(&self) -> Option<&TimeData>;
    fn icon(&self) -> PredefinedIcon;
    fn custom_icon_uuid(&self) -> Option<Uuid>;
    fn tags(&self) -> &[String];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseElementRef<'a> {
    Group(&'a Group),
    Entry(&'a Entry),
}

impl DatabaseElement for DatabaseElementRef<'_> {
    fn uuid(&self) -> Uuid {
        match self {
            Self::Group(group) => group.uuid(),
            Self::Entry(entry) => entry.uuid(),
        }
    }

    fn times(&self) -> Option<&TimeData> {
        match self {
            Self::Group(group) => group.times(),
            Self::Entry(entry) => entry.times(),
        }
    }

    fn icon(&self) -> PredefinedIcon {
        match self {
            Self::Group(group) => group.icon(),
            Self::Entry(entry) => entry.icon(),
        }
    }

    fn custom_icon_uuid(&self) -> Option<Uuid> {
        match self {
            Self::Group(group) => group.custom_icon_uuid(),
            Self::Entry(entry) => entry.custom_icon_uuid(),
        }
    }

    fn tags(&self) -> &[String] {
        match self {
            Self::Group(group) => group.tags(),
            Self::Entry(entry) => entry.tags(),
        }
    }
}
