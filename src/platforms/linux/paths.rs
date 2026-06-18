use std::collections::HashSet;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::app_entry::{AppIcon, AppIconSize};

pub(super) fn application_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(data_home) = data_home() {
        dirs.push(data_home.join("applications"));
        dirs.push(
            data_home
                .join("flatpak")
                .join("exports")
                .join("share")
                .join("applications"),
        );
    }

    for data_dir in data_dirs() {
        dirs.push(data_dir.join("applications"));
    }

    dirs.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));
    dirs.push(PathBuf::from("/var/lib/snapd/desktop/applications"));

    dedupe_paths(dirs)
}

pub(super) fn visit_desktop_files(dir: &Path, visitor: &mut impl FnMut(&Path)) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if is_desktop_file(&path) {
            visitor(&path);
        } else if path.is_dir() {
            visit_desktop_files(&path, visitor);
        }
    }
}

pub(super) fn is_desktop_file(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension == "desktop")
}

pub(super) fn command_exists(command: &str) -> bool {
    let command = Path::new(command);

    if command.components().count() > 1 {
        return is_executable(command);
    }

    env::var_os("PATH")
        .map(split_paths)
        .unwrap_or_default()
        .into_iter()
        .any(|dir| is_executable(&dir.join(command)))
}

fn is_executable(path: &Path) -> bool {
    path.metadata()
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

pub(super) fn resolve_icon_paths(icon: &str) -> Vec<AppIcon> {
    let icon_path = Path::new(icon);

    if icon_path.is_absolute() {
        if icon_path.exists() {
            return vec![AppIcon {
                path: icon_path.to_path_buf(),
                size: icon_size_from_path(icon_path),
            }];
        }

        return Vec::new();
    }

    icon_candidates(icon)
        .into_iter()
        .filter(|icon| icon.path.exists())
        .collect()
}

fn data_home() -> Option<PathBuf> {
    env::var_os("XDG_DATA_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
}

fn data_dirs() -> Vec<PathBuf> {
    env::var_os("XDG_DATA_DIRS")
        .filter(|value| !value.is_empty())
        .map(split_paths)
        .unwrap_or_else(|| {
            vec![
                PathBuf::from("/usr/local/share"),
                PathBuf::from("/usr/share"),
            ]
        })
}

fn split_paths(paths: OsString) -> Vec<PathBuf> {
    env::split_paths(&paths).collect()
}

fn dedupe_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    paths
        .into_iter()
        .filter(|path| seen.insert(path.clone()))
        .collect()
}

fn icon_candidates(icon: &str) -> Vec<AppIcon> {
    let mut roots = Vec::new();

    if let Some(data_home) = data_home() {
        roots.push(data_home);
    }

    roots.extend(data_dirs());

    let extensions = ["svg", "png", "xpm"];

    let mut candidates = Vec::new();

    for root in roots {
        push_icon_candidate(
            &mut candidates,
            root.join("pixmaps").join(icon),
            AppIconSize::Unknown,
        );

        for extension in extensions {
            let path = root.join("pixmaps").join(format!("{icon}.{extension}"));
            let size = icon_size_from_extension(extension);
            push_icon_candidate(&mut candidates, path, size);
        }

        push_theme_icon_candidates(&mut candidates, &root, icon, extensions);
    }

    dedupe_icons(candidates)
}

fn push_theme_icon_candidates(
    candidates: &mut Vec<AppIcon>,
    root: &Path,
    icon: &str,
    extensions: [&str; 3],
) {
    let icons_dir = root.join("icons");
    let Ok(themes) = fs::read_dir(icons_dir) else {
        return;
    };

    for theme in themes.flatten() {
        let theme_path = theme.path();

        if !theme_path.is_dir() {
            continue;
        }

        push_theme_context_icon_candidates(candidates, &theme_path, icon, extensions);
    }
}

fn push_theme_context_icon_candidates(
    candidates: &mut Vec<AppIcon>,
    theme_path: &Path,
    icon: &str,
    extensions: [&str; 3],
) {
    let Ok(size_dirs) = fs::read_dir(theme_path) else {
        return;
    };

    for size_dir in size_dirs.flatten() {
        let size_path = size_dir.path();

        if !size_path.is_dir() {
            continue;
        }

        let size = size_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(icon_size_from_directory)
            .unwrap_or(AppIconSize::Unknown);
        let apps_dir = size_path.join("apps");

        for extension in extensions {
            push_icon_candidate(
                candidates,
                apps_dir.join(format!("{icon}.{extension}")),
                size,
            );
        }
    }
}

fn push_icon_candidate(candidates: &mut Vec<AppIcon>, path: PathBuf, size: AppIconSize) {
    candidates.push(AppIcon { path, size });
}

fn icon_size_from_path(path: &Path) -> AppIconSize {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(icon_size_from_extension)
        .unwrap_or(AppIconSize::Unknown)
}

fn icon_size_from_extension(extension: &str) -> AppIconSize {
    if extension.eq_ignore_ascii_case("svg") {
        AppIconSize::Scalable
    } else {
        AppIconSize::Unknown
    }
}

fn icon_size_from_directory(size: &str) -> AppIconSize {
    if size == "scalable" {
        return AppIconSize::Scalable;
    }

    let Some((width, height)) = size.split_once('x') else {
        return AppIconSize::Unknown;
    };

    match (width.parse::<u32>(), height.parse::<u32>()) {
        (Ok(width), Ok(height)) if width == height => AppIconSize::Pixels(width),
        _ => AppIconSize::Unknown,
    }
}

fn dedupe_icons(icons: Vec<AppIcon>) -> Vec<AppIcon> {
    let mut seen = HashSet::new();
    icons
        .into_iter()
        .filter(|icon| seen.insert(icon.path.clone()))
        .collect()
}
