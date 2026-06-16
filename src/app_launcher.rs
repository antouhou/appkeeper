use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::app_entry::AppEntry;

pub trait AppLauncher {
    fn launch(&self, app: &AppEntry, options: LaunchOptions) -> Result<(), LaunchError>;
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LaunchOptions {
    pub files: Vec<PathBuf>,
    pub urls: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum LaunchError {
    #[error("app does not have a launch command")]
    NotLaunchable,

    #[error("launch argument is not supported: {0}")]
    UnsupportedArgument(&'static str),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordedLaunch {
    pub app: AppEntry,
    pub options: LaunchOptions,
}

#[derive(Clone, Default)]
pub struct MockLauncher {
    launches: Arc<Mutex<Vec<RecordedLaunch>>>,
}

impl MockLauncher {
    pub fn launches(&self) -> Vec<RecordedLaunch> {
        self.launches
            .lock()
            .expect("mock launcher launches lock was poisoned")
            .clone()
    }
}

impl AppLauncher for MockLauncher {
    fn launch(&self, app: &AppEntry, options: LaunchOptions) -> Result<(), LaunchError> {
        self.launches
            .lock()
            .expect("mock launcher launches lock was poisoned")
            .push(RecordedLaunch {
                app: app.clone(),
                options,
            });

        Ok(())
    }
}
