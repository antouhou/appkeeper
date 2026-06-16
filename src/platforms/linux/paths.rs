use std::collections::HashSet;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

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

pub(super) fn resolve_icon_path(icon: &str) -> Option<PathBuf> {
    let icon_path = Path::new(icon);

    if icon_path.is_absolute() {
        return icon_path.exists().then(|| icon_path.to_path_buf());
    }

    icon_candidates(icon)
        .into_iter()
        .find(|candidate| candidate.exists())
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

fn icon_candidates(icon: &str) -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Some(data_home) = data_home() {
        roots.push(data_home);
    }

    roots.extend(data_dirs());

    let sizes = [
        "scalable", "512x512", "256x256", "128x128", "64x64", "48x48", "32x32", "24x24", "16x16",
    ];
    let extensions = ["svg", "png", "xpm"];

    let mut candidates = Vec::new();

    for root in roots {
        candidates.push(root.join("pixmaps").join(icon));

        for extension in extensions {
            candidates.push(root.join("pixmaps").join(format!("{icon}.{extension}")));
        }

        for size in sizes {
            for extension in extensions {
                candidates.push(
                    root.join("icons")
                        .join("hicolor")
                        .join(size)
                        .join("apps")
                        .join(format!("{icon}.{extension}")),
                );
            }
        }
    }

    candidates
}
