use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppEntry {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub install_location: Option<PathBuf>,
    pub icons: Vec<AppIcon>,
    pub categories: Vec<String>,
    pub status: AppStatus,
    pub launch: Option<LaunchCommand>,
}

impl AppEntry {
    pub fn icon_for_size(&self, size: u32) -> Option<&AppIcon> {
        self.icons
            .iter()
            .min_by_key(|icon| icon.size.match_score(size))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppIcon {
    pub path: PathBuf,
    pub size: AppIconSize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppIconSize {
    Pixels(u32),
    Scalable,
    Unknown,
}

impl AppIconSize {
    fn match_score(self, target_size: u32) -> (u8, u32) {
        match self {
            Self::Pixels(size) if size == target_size => (0, 0),
            Self::Scalable => (1, 0),
            Self::Pixels(size) if size > target_size => (2, size - target_size),
            Self::Pixels(size) => (3, target_size - size),
            Self::Unknown => (4, 0),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AppStatus {
    pub is_running: bool,
    pub is_launching: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaunchCommand {
    pub executable: PathBuf,
    pub args: Vec<LaunchArg>,
    pub requires_terminal: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaunchArg {
    Literal(String),
    Template(Vec<LaunchArgPart>),
    File,
    Files,
    Url,
    Urls,
    AppName,
    Icon(String),
    DesktopFile(PathBuf),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaunchArgPart {
    Literal(String),
    File,
    Url,
    AppName,
    DesktopFile(PathBuf),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn picks_best_icon_for_requested_size() {
        let app = AppEntry {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: None,
            description: None,
            publisher: None,
            install_location: None,
            icons: vec![
                AppIcon {
                    path: PathBuf::from("unknown.xpm"),
                    size: AppIconSize::Unknown,
                },
                AppIcon {
                    path: PathBuf::from("64.png"),
                    size: AppIconSize::Pixels(64),
                },
                AppIcon {
                    path: PathBuf::from("32.png"),
                    size: AppIconSize::Pixels(32),
                },
                AppIcon {
                    path: PathBuf::from("icon.svg"),
                    size: AppIconSize::Scalable,
                },
            ],
            categories: Vec::new(),
            status: AppStatus::default(),
            launch: None,
        };

        assert_eq!(
            app.icon_for_size(32).map(|icon| icon.path.as_path()),
            Some(Path::new("32.png"))
        );
        assert_eq!(
            app.icon_for_size(48).map(|icon| icon.path.as_path()),
            Some(Path::new("icon.svg"))
        );
    }
}
