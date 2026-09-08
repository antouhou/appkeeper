use super::catalog::{self, AppCatalog};
use super::{LinuxAppProvider, SearchPaths};
use crate::AppId;
use crate::app_entry::LaunchArg;
use crate::app_provider::{AppProvider, AppProviderEvent};
use std::ffi::OsString;
use std::fs;
use std::os::unix;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::slice;
use tempfile::TempDir;

pub(super) const SYSTEM: &str =
    "[Desktop Entry]\nType=Application\nName=System\nExec=/bin/true %k\n";
pub(super) const USER: &str = "[Desktop Entry]\nType=Application\nName=User\nExec=/bin/true %k\n";

pub(super) fn write_entry(root: &Path, relative: &str, contents: &str) {
    let path = root.join("applications").join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

pub(super) fn id(value: &str) -> AppId {
    AppId::from_parts("freedesktop", value)
}

fn load(roots: &[PathBuf]) -> AppCatalog {
    catalog::load_apps(&SearchPaths::from_data_dirs(roots.to_vec())).unwrap()
}

#[test]
fn nested_desktop_files_keep_the_standard_id() {
    let directory = TempDir::new().unwrap();
    write_entry(directory.path(), "foo/bar.desktop", USER);
    write_entry(directory.path(), "other/bar.desktop", SYSTEM);
    let apps = load(&[directory.path().to_path_buf()]);
    assert_eq!(apps.len(), 2);
    assert_eq!(apps[&id("foo-bar.desktop")].name, "User");
    assert_eq!(apps[&id("other-bar.desktop")].name, "System");
    assert!(!apps.contains_key(&id("bar")));
}

#[test]
fn precedence_exposes_only_one_entry_per_id() {
    let directory = TempDir::new().unwrap();
    let user = directory.path().join("user");
    let system = directory.path().join("system");
    write_entry(&user, "app.desktop", USER);
    write_entry(&system, "app.desktop", SYSTEM);
    let roots = [user, system];
    let provider = LinuxAppProvider::with_paths(SearchPaths::from_data_dirs(roots.to_vec()));
    let app = provider.entry(&id("app.desktop")).unwrap();
    assert_eq!(provider.list().as_slice(), slice::from_ref(&app));
    assert_eq!(app.name, "User");
    assert_eq!(
        app.launch_command.unwrap().args,
        [LaunchArg::DesktopFile(
            roots[0].join("applications/app.desktop")
        )]
    );
    assert!(
        provider
            .entry(&AppId::from_parts("mock", "app.desktop"))
            .is_none()
    );
    let reversed = load(&[roots[1].clone(), roots[0].clone()]);
    assert_eq!(reversed[&id("app.desktop")].name, "System");
}

#[test]
fn hidden_override_suppresses_the_system_entry() {
    let directory = TempDir::new().unwrap();
    let user = directory.path().join("user");
    let system = directory.path().join("system");
    write_entry(&user, "app.desktop", "[Desktop Entry]\nHidden=true\n");
    write_entry(&system, "app.desktop", SYSTEM);
    assert!(load(&[user, system]).is_empty());
}

#[test]
fn unlisted_or_invalid_winners_do_not_fall_back() {
    let directory = TempDir::new().unwrap();
    let user = directory.path().join("user");
    let system = directory.path().join("system");
    write_entry(&system, "app.desktop", SYSTEM);
    for contents in [
        "[Desktop Entry]\nType=Application\nName=User\nNoDisplay=true\n",
        "[Desktop Entry]\nType=Application\nName=User\nOnlyShowIn=;\n",
        "[Desktop Entry]\nType=Application\nName=User\nTryExec=/appkeeper-nonexistent-test-command\n",
        "[Desktop Entry]\nType=Link\nName=User\n",
        "[Desktop Entry]\nType=Application\n",
        "invalid desktop entry",
    ] {
        write_entry(&user, "app.desktop", contents);
        assert!(
            load(&[user.clone(), system.clone()]).is_empty(),
            "{contents}"
        );
    }
    fs::remove_file(user.join("applications/app.desktop")).unwrap();
    unix::fs::symlink(
        directory.path().join("nonexistent"),
        user.join("applications/app.desktop"),
    )
    .unwrap();
    assert!(load(&[user, system]).is_empty());
}

#[test]
fn collision_selection_is_sorted_by_relative_path_bytes() {
    let directory = TempDir::new().unwrap();
    write_entry(directory.path(), "foo/bar.desktop", SYSTEM);
    write_entry(directory.path(), "foo-bar.desktop", USER);
    let roots = [directory.path().to_path_buf()];
    let apps = load(&roots);
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[&id("foo-bar.desktop")].name, "User");
    fs::remove_file(directory.path().join("applications/foo-bar.desktop")).unwrap();
    assert_eq!(load(&roots)[&id("foo-bar.desktop")].name, "System");
}

#[test]
fn effective_changes_cover_overrides_fallback_and_shadowed_updates() {
    let directory = TempDir::new().unwrap();
    let user = directory.path().join("user");
    let system = directory.path().join("system");
    let roots = [user.clone(), system.clone()];
    write_entry(&system, "app.desktop", SYSTEM);
    let system_apps = load(&roots);
    assert_eq!(
        catalog::changes(&AppCatalog::new(), &system_apps),
        [AppProviderEvent::Added]
    );
    write_entry(&user, "app.desktop", USER);
    let user_apps = load(&roots);
    assert_eq!(
        catalog::changes(&system_apps, &user_apps),
        [AppProviderEvent::EntryUpdated]
    );
    write_entry(
        &system,
        "app.desktop",
        &SYSTEM.replace("System", "Updated system"),
    );
    assert!(catalog::changes(&user_apps, &load(&roots)).is_empty());
    fs::remove_file(user.join("applications/app.desktop")).unwrap();
    let fallback = load(&roots);
    assert_eq!(fallback[&id("app.desktop")].name, "Updated system");
    assert_eq!(
        catalog::changes(&user_apps, &fallback),
        [AppProviderEvent::EntryUpdated]
    );
    write_entry(&user, "app.desktop", "[Desktop Entry]\nHidden=true\n");
    let hidden = load(&roots);
    assert_eq!(
        catalog::changes(&fallback, &hidden),
        [AppProviderEvent::Removed]
    );
    fs::remove_file(user.join("applications/app.desktop")).unwrap();
    assert_eq!(
        catalog::changes(&hidden, &load(&roots)),
        [AppProviderEvent::Added]
    );
    fs::remove_file(system.join("applications/app.desktop")).unwrap();
    assert_eq!(
        catalog::changes(&fallback, &load(&roots)),
        [AppProviderEvent::Removed]
    );
}

#[test]
fn symlink_ids_use_the_exported_location_and_directory_cycles_terminate() {
    let directory = TempDir::new().unwrap();
    let exported = directory.path().join("exports");
    let applications = exported.join("applications");
    fs::create_dir_all(&applications).unwrap();
    let target = directory.path().join("installed.desktop");
    fs::write(&target, USER).unwrap();
    unix::fs::symlink(target, applications.join("org.example.App.desktop")).unwrap();
    unix::fs::symlink(&applications, applications.join("loop")).unwrap();
    let apps = load(&[exported]);
    assert_eq!(apps.len(), 1);
    let app = &apps[&id("org.example.App.desktop")];
    assert_eq!(
        app.launch_command.as_ref().unwrap().args,
        [LaunchArg::DesktopFile(
            applications.join("org.example.App.desktop")
        )]
    );
}

#[test]
fn non_utf8_filenames_do_not_collide_with_unicode_replacement_characters() {
    let directory = TempDir::new().unwrap();
    write_entry(directory.path(), "bad-�.desktop", USER);
    let invalid = OsString::from_vec(b"bad-\xff.desktop".to_vec());
    fs::write(directory.path().join("applications").join(invalid), SYSTEM).unwrap();
    let apps = load(&[directory.path().to_path_buf()]);
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[&id("bad-�.desktop")].name, "User");
}

#[test]
fn directory_scan_failures_do_not_produce_partial_catalogs() {
    let directory = TempDir::new().unwrap();
    let first = directory.path().join("first");
    let broken = directory.path().join("broken");
    write_entry(&first, "app.desktop", USER);
    fs::create_dir_all(&broken).unwrap();
    fs::write(broken.join("applications"), "not a directory").unwrap();
    assert!(catalog::load_apps(&SearchPaths::from_data_dirs(vec![first, broken])).is_err());
}
