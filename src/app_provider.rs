use crate::app_entry::AppEntry;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppProviderEvent {
    Added,
    Removed,
    EntryUpdated,
}

pub trait AppProvider {
    fn list(&self) -> Vec<AppEntry>;
    fn subscribe(&mut self, cb: fn(AppProviderEvent));
}
