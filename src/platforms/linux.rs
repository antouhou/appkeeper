use crate::app_entry::AppEntry;
use crate::app_provider::{AppProvider, AppProviderEvent};

pub struct LinuxAppProvider;

impl AppProvider for LinuxAppProvider {
    fn list(&self) -> Vec<AppEntry> {
        vec![]
    }

    fn subscribe(&mut self, _cb: fn(AppProviderEvent)) {}

    fn rescan(&mut self) {}
}

impl LinuxAppProvider {
    pub fn new() -> LinuxAppProvider {
        LinuxAppProvider {}
    }
}
