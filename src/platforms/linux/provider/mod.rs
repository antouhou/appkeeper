use super::paths::SearchPaths;
use crate::AppId;
use crate::app_entry::AppEntry;
use crate::app_provider::{AppProvider, AppProviderEvent};
use catalog::AppCatalog;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, RwLock};
use watcher::CatalogWatcher;

mod catalog;

mod watcher;

type AppCache = Arc<RwLock<AppCatalog>>;

type AppCallback = Arc<Mutex<dyn FnMut(AppProviderEvent) + Send>>;
type AppCallbacks = Arc<Mutex<Vec<AppCallback>>>;

pub struct LinuxAppProvider {
    paths: Arc<SearchPaths>,
    apps: AppCache,
    callbacks: AppCallbacks,
    watcher: Option<CatalogWatcher>,
}

impl LinuxAppProvider {
    fn with_paths(paths: SearchPaths) -> Self {
        let apps = catalog::load_apps(&paths).unwrap_or_else(|error| {
            tracing::warn!(%error, "cannot load application catalog");
            AppCatalog::new()
        });
        Self {
            paths: Arc::new(paths),
            apps: Arc::new(RwLock::new(apps)),
            callbacks: Arc::new(Mutex::new(Vec::new())),
            watcher: None,
        }
    }

    fn start_watcher(&mut self) {
        if self.watcher.is_some() {
            return;
        }
        let apps = Arc::clone(&self.apps);
        let callbacks = Arc::clone(&self.callbacks);
        match CatalogWatcher::start(Arc::clone(&self.paths), move |next, stopping| {
            let events = {
                let mut apps = apps
                    .write()
                    .expect("linux app provider apps lock was poisoned");
                if stopping.load(Ordering::Acquire) {
                    return;
                }
                let events = catalog::changes(&apps, &next);
                *apps = next;
                events
            };
            let callbacks = callbacks
                .lock()
                .expect("linux app provider callbacks lock was poisoned")
                .clone();
            for event in events {
                for callback in &callbacks {
                    if stopping.load(Ordering::Acquire) {
                        return;
                    }
                    let mut callback = callback
                        .lock()
                        .expect("linux app provider callback lock was poisoned");
                    callback(event);
                }
            }
        }) {
            Ok(watcher) => self.watcher = Some(watcher),
            Err(error) => tracing::warn!(%error, "cannot watch application directories"),
        }
    }

    pub fn new() -> Self {
        Self::with_paths(SearchPaths::from_env())
    }
}

impl AppProvider for LinuxAppProvider {
    fn entry(&self, id: &AppId) -> Option<AppEntry> {
        self.apps
            .read()
            .expect("linux app provider apps lock was poisoned")
            .get(id)
            .cloned()
    }

    fn list(&self) -> Vec<AppEntry> {
        let mut apps = self
            .apps
            .read()
            .expect("linux app provider apps lock was poisoned")
            .values()
            .cloned()
            .collect::<Vec<_>>();
        apps.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then_with(|| left.id.cmp(&right.id))
        });
        apps
    }

    fn subscribe(&mut self, callback: impl FnMut(AppProviderEvent) + Send + 'static) {
        self.callbacks
            .lock()
            .expect("linux app provider callbacks lock was poisoned")
            .push(Arc::new(Mutex::new(callback)));
        self.start_watcher();
    }
}

#[cfg(test)]
mod tests;
