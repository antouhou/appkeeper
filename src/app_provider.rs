use crate::AppId;
use crate::app_entry::AppEntry;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppProviderEvent {
    Added(AppEntry),
    Removed(AppId),
    EntryUpdated(AppEntry),
}

pub trait AppProvider {
    fn list(&self) -> Vec<AppEntry>;
    /// Looks up an app by ID. Returns `None` for unsupported IDs or missing apps.
    fn entry(&self, id: &AppId) -> Option<AppEntry>;
    /// Registers a callback with the full entry for additions and updates,
    /// or the app ID for removals. The catalog is updated before callbacks run.
    fn subscribe(&mut self, callback: impl FnMut(AppProviderEvent) + Send + 'static);
}
