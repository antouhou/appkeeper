use crate::AppId;
use crate::app_entry::AppEntry;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppProviderEvent {
    Added,
    Removed,
    EntryUpdated,
}

pub trait AppProvider {
    fn list(&self) -> Vec<AppEntry>;
    /// Looks up an app by ID. Returns `None` for unsupported IDs or missing apps.
    fn entry(&self, id: &AppId) -> Option<AppEntry>;
    fn subscribe(&mut self, cb: fn(AppProviderEvent));
}
