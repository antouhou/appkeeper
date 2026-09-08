use super::super::paths::{self, SearchPaths};
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn configured_order_precedes_deduplicated_fallbacks() {
    let paths = SearchPaths::from_environment(
        Some(OsStr::new("/users/test/data")),
        Some(OsStr::new(
            "/opt/share:/var/lib/flatpak/exports/share:/usr/share:/opt/share/",
        )),
        None,
    );
    assert_eq!(
        paths.data_dirs(),
        [
            PathBuf::from("/users/test/data"),
            PathBuf::from("/opt/share"),
            PathBuf::from("/var/lib/flatpak/exports/share"),
            PathBuf::from("/usr/share"),
            PathBuf::from("/users/test/data/flatpak/exports/share"),
            PathBuf::from("/var/lib/snapd/desktop"),
        ]
    );
    assert_eq!(
        paths.application_dirs(),
        paths
            .data_dirs()
            .iter()
            .map(|root| root.join("applications"))
            .collect::<Vec<_>>()
    );
}

#[test]
fn absent_or_empty_environment_uses_defaults_and_keeps_flatpak_discovery() {
    for value in [None, Some(OsStr::new(""))] {
        let paths = SearchPaths::from_environment(value, value, Some(OsStr::new("/users/test")));
        assert_eq!(
            paths.data_dirs(),
            [
                PathBuf::from("/users/test/.local/share"),
                PathBuf::from("/usr/local/share"),
                PathBuf::from("/usr/share"),
                PathBuf::from("/users/test/.local/share/flatpak/exports/share"),
                PathBuf::from("/var/lib/flatpak/exports/share"),
                PathBuf::from("/var/lib/snapd/desktop"),
            ]
        );
    }
}

#[test]
fn relative_environment_paths_are_ignored() {
    let paths = SearchPaths::from_environment(
        Some(OsStr::new("relative/home")),
        Some(OsStr::new("relative::/absolute/share:./another")),
        Some(OsStr::new("/users/test")),
    );
    assert_eq!(
        paths.data_dirs()[0],
        PathBuf::from("/users/test/.local/share")
    );
    assert_eq!(paths.data_dirs()[1], PathBuf::from("/absolute/share"));
    assert!(paths.data_dirs().iter().all(|root| root.is_absolute()));
    assert!(!paths.data_dirs().contains(&PathBuf::from("/usr/share")));
    let no_home = SearchPaths::from_environment(
        None,
        Some(OsStr::new("relative")),
        Some(OsStr::new("relative")),
    );
    assert_eq!(no_home.data_dirs().len(), 2);
}

#[test]
fn exported_icons_use_the_same_fallback_roots_as_applications() {
    let directory = TempDir::new().unwrap();
    let data_home = directory.path().join("data");
    let configured = directory.path().join("configured");
    let paths = SearchPaths::from_environment(
        Some(data_home.as_os_str()),
        Some(configured.as_os_str()),
        None,
    );
    let export = data_home.join("flatpak/exports/share");
    let icon = export.join("icons/hicolor/scalable/apps/org.appkeeper.FallbackTest.svg");
    fs::create_dir_all(icon.parent().unwrap()).unwrap();
    fs::write(&icon, "<svg/>").unwrap();
    assert!(
        paths
            .application_dirs()
            .contains(&export.join("applications"))
    );
    let icons = paths::resolve_icon_paths("org.appkeeper.FallbackTest", paths.data_dirs());
    assert_eq!(icons.len(), 1);
    assert_eq!(icons[0].path, icon);
    let configured_icon =
        configured.join("icons/hicolor/scalable/apps/org.appkeeper.FallbackTest.svg");
    fs::create_dir_all(configured_icon.parent().unwrap()).unwrap();
    fs::write(&configured_icon, "<svg/>").unwrap();
    let icons = paths::resolve_icon_paths("org.appkeeper.FallbackTest", paths.data_dirs());
    assert_eq!(icons[0].path, configured_icon);
    assert_eq!(icons[1].path, icon);
}
