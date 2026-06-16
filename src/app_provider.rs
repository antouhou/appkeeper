use crate::app_entry::AppEntry;

pub enum AppProviderEvent {
    Added,
    Removed,
    EntryUpdated,
}

pub trait AppProvider {
    fn list(&self) -> Vec<AppEntry>;
    fn subscribe(&mut self, cb: fn(AppProviderEvent));
    fn rescan(&mut self);
}

