use crate::app_entry::AppEntry;
use crate::app_provider::{AppProvider, AppProviderEvent};

#[allow(dead_code)]
pub struct MockProvider {}

impl AppProvider for MockProvider {
    fn list(&self) -> Vec<AppEntry> {
        unimplemented!()
    }

    fn subscribe(&mut self, _cb: fn(AppProviderEvent)) {
        unimplemented!()
    }

    fn rescan(&mut self) {
        unimplemented!()
    }
}
