#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(usize)]
pub enum PredefinedIcon {
    #[default]
    Key = 0,
    World = 1,
    Warning = 2,
    NetworkServer = 3,
    MarkedDirectory = 4,
    UserCommunication = 5,
    Parts = 6,
    Notepad = 7,
    WorldSocket = 8,
    Identity = 9,
    PaperReady = 10,
    Digicam = 11,
    IRCommunication = 12,
    MultiKeys = 13,
    Energy = 14,
    Scanner = 15,
    WorldStar = 16,
    CDRom = 17,
    Monitor = 18,
    Email = 19,
    Configuration = 20,
    ClipboardReady = 21,
    PaperNew = 22,
    Screen = 23,
    EnergyCareful = 24,
    EmailBox = 25,
    Disk = 26,
    Drive = 27,
    PaperQ = 28,
    TerminalEncrypted = 29,
    Console = 30,
    Printer = 31,
    ProgramIcons = 32,
    Run = 33,
    Settings = 34,
    WorldComputer = 35,
    Archive = 36,
    HomeBanking = 37,
    DriveWindows = 38,
    Clock = 39,
    EmailSearch = 40,
    PaperFlag = 41,
    Memory = 42,
    TrashBin = 43,
    Note = 44,
    Expired = 45,
    Info = 46,
    Package = 47,
    Folder = 48,
    FolderOpen = 49,
    FolderPackage = 50,
    LockOpen = 51,
    PaperLocked = 52,
    Checked = 53,
    Pen = 54,
    Thumbnail = 55,
    Book = 56,
    List = 57,
    UserKey = 58,
    Tool = 59,
    Home = 60,
    Star = 61,
    Tux = 62,
    Feather = 63,
    Apple = 64,
    Wiki = 65,
    Money = 66,
    Certificate = 67,
    BlackBerry = 68,
}

impl PredefinedIcon {
    pub const ALL: [Self; 69] = [
        Self::Key,
        Self::World,
        Self::Warning,
        Self::NetworkServer,
        Self::MarkedDirectory,
        Self::UserCommunication,
        Self::Parts,
        Self::Notepad,
        Self::WorldSocket,
        Self::Identity,
        Self::PaperReady,
        Self::Digicam,
        Self::IRCommunication,
        Self::MultiKeys,
        Self::Energy,
        Self::Scanner,
        Self::WorldStar,
        Self::CDRom,
        Self::Monitor,
        Self::Email,
        Self::Configuration,
        Self::ClipboardReady,
        Self::PaperNew,
        Self::Screen,
        Self::EnergyCareful,
        Self::EmailBox,
        Self::Disk,
        Self::Drive,
        Self::PaperQ,
        Self::TerminalEncrypted,
        Self::Console,
        Self::Printer,
        Self::ProgramIcons,
        Self::Run,
        Self::Settings,
        Self::WorldComputer,
        Self::Archive,
        Self::HomeBanking,
        Self::DriveWindows,
        Self::Clock,
        Self::EmailSearch,
        Self::PaperFlag,
        Self::Memory,
        Self::TrashBin,
        Self::Note,
        Self::Expired,
        Self::Info,
        Self::Package,
        Self::Folder,
        Self::FolderOpen,
        Self::FolderPackage,
        Self::LockOpen,
        Self::PaperLocked,
        Self::Checked,
        Self::Pen,
        Self::Thumbnail,
        Self::Book,
        Self::List,
        Self::UserKey,
        Self::Tool,
        Self::Home,
        Self::Star,
        Self::Tux,
        Self::Feather,
        Self::Apple,
        Self::Wiki,
        Self::Money,
        Self::Certificate,
        Self::BlackBerry,
    ];

    pub const fn ordinal(self) -> usize {
        self as usize
    }

    pub fn from_ordinal(ordinal: usize) -> Option<Self> {
        Self::ALL.get(ordinal).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::PredefinedIcon;

    #[test]
    fn ordinals_match_kotlin_enum_order() {
        assert_eq!(PredefinedIcon::Key.ordinal(), 0);
        assert_eq!(PredefinedIcon::TrashBin.ordinal(), 43);
        assert_eq!(PredefinedIcon::Folder.ordinal(), 48);
        assert_eq!(PredefinedIcon::BlackBerry.ordinal(), 68);
    }

    #[test]
    fn parses_known_ordinals() {
        assert_eq!(PredefinedIcon::from_ordinal(0), Some(PredefinedIcon::Key));
        assert_eq!(
            PredefinedIcon::from_ordinal(48),
            Some(PredefinedIcon::Folder)
        );
        assert_eq!(
            PredefinedIcon::from_ordinal(68),
            Some(PredefinedIcon::BlackBerry)
        );
        assert_eq!(PredefinedIcon::from_ordinal(69), None);
    }
}
