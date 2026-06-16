use std::path::PathBuf;

pub struct AppEntry {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub install_location: Option<PathBuf>,
    pub icon_path: Option<PathBuf>,
    pub categories: Vec<String>,
    pub launch: Option<LaunchCommand>,
}

pub struct LaunchCommand {
    pub executable: PathBuf,
    pub args: Vec<LaunchArg>,
    pub requires_terminal: bool,
}

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
