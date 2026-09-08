use super::super::tests::{self, SYSTEM, USER};
use super::super::watcher;
use super::{AppCatalog, CatalogWatcher, SearchPaths};
use notify::event::{AccessKind, AccessMode, Flag};
use notify::{Event, EventKind};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};
use tempfile::TempDir;

fn start(roots: Vec<PathBuf>) -> (CatalogWatcher, Receiver<AppCatalog>) {
    let (sender, receiver) = mpsc::channel();
    let watcher = CatalogWatcher::start(
        Arc::new(SearchPaths::from_data_dirs(roots)),
        move |apps, _| {
            sender.send(apps).unwrap();
        },
    )
    .unwrap();
    (watcher, receiver)
}

#[track_caller]
fn receive_catalog(
    receiver: &Receiver<AppCatalog>,
    matches: impl Fn(&AppCatalog) -> bool,
) -> AppCatalog {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let apps = receiver
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .expect("catalog notification before deadline");
        if matches(&apps) {
            return apps;
        }
    }
}

#[test]
fn native_notifications_cover_initially_missing_roots_overrides_and_fallback() {
    let directory = TempDir::new().unwrap();
    let user = directory.path().join("missing/deep/user");
    let system = directory.path().join("system");
    tests::write_entry(&system, "app.desktop", SYSTEM);
    let (watcher, receiver) = start(vec![user.clone(), system.clone()]);
    receive_catalog(&receiver, |apps| {
        apps.get(&tests::id("app.desktop"))
            .is_some_and(|app| app.name == "System")
    });
    tests::write_entry(&user, "app.desktop", USER);
    receive_catalog(&receiver, |apps| {
        apps.get(&tests::id("app.desktop"))
            .is_some_and(|app| app.name == "User")
    });
    tests::write_entry(&user, "app.desktop", "[Desktop Entry]\nHidden=true\n");
    receive_catalog(&receiver, AppCatalog::is_empty);
    fs::remove_file(user.join("applications/app.desktop")).unwrap();
    receive_catalog(&receiver, |apps| {
        apps.get(&tests::id("app.desktop"))
            .is_some_and(|app| app.name == "System")
    });
    drop(watcher);
    // The publishing closure is released when the worker exits.
    while receiver.recv().is_ok() {}
}

#[test]
fn directory_renames_and_root_recreation_refresh_the_catalog() {
    let directory = TempDir::new().unwrap();
    let root = directory.path().join("data");
    tests::write_entry(&root, "old/app.desktop", USER);
    let (watcher, receiver) = start(vec![root.clone()]);
    receive_catalog(&receiver, |apps| {
        apps.contains_key(&tests::id("old-app.desktop"))
    });
    fs::rename(root.join("applications/old"), root.join("applications/new")).unwrap();
    receive_catalog(&receiver, |apps| {
        apps.contains_key(&tests::id("new-app.desktop"))
            && !apps.contains_key(&tests::id("old-app.desktop"))
    });
    fs::remove_dir_all(&root).unwrap();
    receive_catalog(&receiver, AppCatalog::is_empty);
    tests::write_entry(&root, "restored.desktop", SYSTEM);
    receive_catalog(&receiver, |apps| {
        apps.contains_key(&tests::id("restored.desktop"))
    });
    tests::write_entry(&root, "later.desktop", USER);
    receive_catalog(&receiver, |apps| apps.len() == 2);
    drop(watcher);
}

#[test]
fn atomic_desktop_file_replacements_are_observed() {
    let directory = TempDir::new().unwrap();
    let root = directory.path().to_path_buf();
    tests::write_entry(&root, "app.desktop", SYSTEM);
    let (_watcher, receiver) = start(vec![root.clone()]);
    receive_catalog(&receiver, |apps| {
        apps.get(&tests::id("app.desktop"))
            .is_some_and(|app| app.name == "System")
    });
    let temporary = root.join("replacement.tmp");
    fs::write(&temporary, USER).unwrap();
    fs::rename(temporary, root.join("applications/app.desktop")).unwrap();
    receive_catalog(&receiver, |apps| {
        apps.get(&tests::id("app.desktop"))
            .is_some_and(|app| app.name == "User")
    });
}

#[test]
fn notification_filter_keeps_directory_and_overflow_events_without_read_feedback() {
    let paths = SearchPaths::from_data_dirs(vec![PathBuf::from("/data")]);
    let directory =
        Event::new(EventKind::Any).add_path(PathBuf::from("/data/applications/subdirectory"));
    let ancestor = Event::new(EventKind::Any).add_path(PathBuf::from("/data"));
    let unrelated = Event::new(EventKind::Any).add_path(PathBuf::from("/data/unrelated"));
    let opened = Event::new(EventKind::Access(AccessKind::Open(AccessMode::Read)))
        .add_path(PathBuf::from("/data/applications/app.desktop"));
    let written = Event::new(EventKind::Access(AccessKind::Close(AccessMode::Write)))
        .add_path(PathBuf::from("/data/applications/app.desktop"));
    let overflow = Event::new(EventKind::Other).set_flag(Flag::Rescan);
    assert!(watcher::relevant(&directory, &paths));
    assert!(watcher::relevant(&ancestor, &paths));
    assert!(watcher::relevant(&written, &paths));
    assert!(watcher::relevant(&overflow, &paths));
    assert!(!watcher::relevant(&unrelated, &paths));
    assert!(!watcher::relevant(&opened, &paths));
}
