use crate::AppId;
use std::path::PathBuf;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppIcon {
    pub path: PathBuf,
    pub size: AppIconSize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AppStatus {
    pub is_running: bool,
    pub is_launching: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaunchArgPart {
    Literal(String),
    File,
    Url,
    AppName,
    DesktopFile(PathBuf),
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
pub struct LaunchCommand {
    pub executable: PathBuf,
    pub args: Vec<LaunchArg>,
    pub requires_terminal: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppEntry {
    pub id: AppId,
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub install_location: Option<PathBuf>,
    pub icons: Vec<AppIcon>,
    pub categories: Vec<String>,
    pub status: AppStatus,
    pub launch_command: Option<LaunchCommand>,
}

impl AppEntry {
    pub fn icon_for_size(&self, size: u32) -> Option<&AppIcon> {
        self.icons
            .iter()
            .min_by_key(|icon| icon.size.match_score(size))
    }
}

#[cfg(test)]
mod tests;
