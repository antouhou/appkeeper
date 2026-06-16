use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::app_entry::AppEntry;
use crate::app_provider::{AppProvider, AppProviderEvent};

use super::desktop_entry::desktop_file_to_app;
use super::paths::{application_dirs, is_desktop_file, visit_desktop_files};

type AppCache = Arc<RwLock<HashMap<PathBuf, AppEntry>>>;
type AppCallbacks = Arc<Mutex<Vec<fn(AppProviderEvent)>>>;

pub struct LinuxAppProvider {
    apps: AppCache,
    callbacks: AppCallbacks,
    watcher: Option<RecommendedWatcher>,
}

impl AppProvider for LinuxAppProvider {
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

    fn subscribe(&mut self, cb: fn(AppProviderEvent)) {
        self.callbacks
            .lock()
            .expect("linux app provider callbacks lock was poisoned")
            .push(cb);

        self.start_watcher();
    }
}

impl LinuxAppProvider {
    pub fn new() -> Self {
        Self {
            apps: Arc::new(RwLock::new(load_initial_apps())),
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
        let Ok(mut watcher) = notify::recommended_watcher(move |event| {
            if let Ok(event) = event {
                handle_desktop_file_event(&apps, &callbacks, event);
            }
        }) else {
            return;
        };

        for dir in application_dirs() {
            if dir.is_dir() {
                let _ = watcher.watch(&dir, RecursiveMode::Recursive);
            }
        }

        self.watcher = Some(watcher);
    }
}

fn load_initial_apps() -> HashMap<PathBuf, AppEntry> {
    let mut apps = HashMap::new();

    for dir in application_dirs() {
        visit_desktop_files(&dir, &mut |path| {
            if let Some(app) = desktop_file_to_app(path) {
                apps.insert(path.to_path_buf(), app);
            }
        });
    }

    apps
}

fn handle_desktop_file_event(apps: &AppCache, callbacks: &AppCallbacks, event: Event) {
    if matches!(event.kind, EventKind::Access(_)) {
        return;
    }

    for path in event.paths.into_iter().filter(|path| is_desktop_file(path)) {
        let provider_event = update_desktop_file_entry(apps, &path);

        if let Some(provider_event) = provider_event {
            emit(callbacks, provider_event);
        }
    }
}

fn update_desktop_file_entry(apps: &AppCache, path: &Path) -> Option<AppProviderEvent> {
    let next_app = if path.exists() {
        desktop_file_to_app(path)
    } else {
        None
    };

    let mut apps = apps
        .write()
        .expect("linux app provider apps lock was poisoned");

    match (apps.get(path), next_app) {
        (None, Some(app)) => {
            apps.insert(path.to_path_buf(), app);
            Some(AppProviderEvent::Added)
        }
        (Some(current), Some(app)) if current != &app => {
            apps.insert(path.to_path_buf(), app);
            Some(AppProviderEvent::EntryUpdated)
        }
        (Some(_), None) => {
            apps.remove(path);
            Some(AppProviderEvent::Removed)
        }
        _ => None,
    }
}

fn emit(callbacks: &AppCallbacks, event: AppProviderEvent) {
    let callbacks = callbacks
        .lock()
        .expect("linux app provider callbacks lock was poisoned")
        .clone();

    for callback in callbacks {
        callback(event);
    }
}
