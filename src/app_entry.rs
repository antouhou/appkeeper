use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppEntry {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub install_location: Option<PathBuf>,
    pub icon_path: Option<PathBuf>,
    pub categories: Vec<String>,
    pub status: AppStatus,
    pub launch: Option<LaunchCommand>,
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
    File,
    Files,
    Url,
    Urls,
    AppName,
    Icon,
    DesktopFile,
}
