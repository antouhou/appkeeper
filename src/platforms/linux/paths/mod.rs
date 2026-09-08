use crate::app_entry::{AppIcon, AppIconSize};
use std::collections::HashSet;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

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

fn is_executable(path: &Path) -> bool {
    path.metadata()
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
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

fn icon_size_from_path(path: &Path) -> AppIconSize {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(icon_size_from_extension)
        .unwrap_or(AppIconSize::Unknown)
}

fn push_icon_candidate(candidates: &mut Vec<AppIcon>, path: PathBuf, size: AppIconSize) {
    candidates.push(AppIcon { path, size });
}

fn dedupe_icons(icons: Vec<AppIcon>) -> Vec<AppIcon> {
    let mut seen = HashSet::new();
    icons
        .into_iter()
        .filter(|icon| seen.insert(icon.path.clone()))
        .collect()
}

fn push_pixmap_icon_candidates(
    candidates: &mut Vec<AppIcon>,
    root: &Path,
    icon: &str,
    extensions: [&str; 3],
) {
    push_icon_candidate(
        candidates,
        root.join("pixmaps").join(icon),
        AppIconSize::Unknown,
    );

    for extension in extensions {
        let path = root.join("pixmaps").join(format!("{icon}.{extension}"));
        let size = icon_size_from_extension(extension);
        push_icon_candidate(candidates, path, size);
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

fn push_hicolor_icon_candidates(
    candidates: &mut Vec<AppIcon>,
    root: &Path,
    icon: &str,
    extensions: [&str; 3],
) {
    push_theme_context_icon_candidates(
        candidates,
        &root.join("icons").join("hicolor"),
        icon,
        extensions,
    );
}

fn icon_candidates_in_roots(icon: &str, roots: &[PathBuf], extensions: [&str; 3]) -> Vec<AppIcon> {
    let mut candidates = Vec::new();

    for root in roots {
        push_hicolor_icon_candidates(&mut candidates, root, icon, extensions);
    }

    for root in roots {
        push_pixmap_icon_candidates(&mut candidates, root, icon, extensions);
    }

    dedupe_icons(candidates)
}

pub(super) fn resolve_icon_paths(icon: &str, roots: &[PathBuf]) -> Vec<AppIcon> {
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

    icon_candidates_in_roots(icon, roots, ["svg", "png", "xpm"])
        .into_iter()
        .filter(|icon| icon.path.exists())
        .collect()
}

/// Search directories for desktop entries and icons, fixed at construction.
pub(super) struct SearchPaths {
    data_dirs: Vec<PathBuf>,
    application_dirs: Vec<PathBuf>,
}

impl SearchPaths {
    pub(super) fn application_dirs(&self) -> &[PathBuf] {
        &self.application_dirs
    }

    pub(super) fn data_dirs(&self) -> &[PathBuf] {
        &self.data_dirs
    }

    pub(super) fn from_data_dirs(data_dirs: Vec<PathBuf>) -> Self {
        let data_dirs = dedupe_paths(data_dirs);
        let application_dirs = data_dirs
            .iter()
            .map(|dir| dir.join("applications"))
            .collect();
        Self {
            data_dirs,
            application_dirs,
        }
    }

    fn from_environment(
        data_home: Option<&OsStr>,
        data_dirs: Option<&OsStr>,
        home: Option<&OsStr>,
    ) -> Self {
        let data_home = data_home
            .map(Path::new)
            .filter(|path| path.is_absolute())
            .map(Path::to_path_buf)
            .or_else(|| {
                home.map(Path::new)
                    .filter(|path| path.is_absolute())
                    .map(|home| home.join(".local/share"))
            });
        let mut roots = Vec::new();
        roots.extend(data_home.iter().cloned());
        match data_dirs.filter(|value| !value.is_empty()) {
            Some(dirs) => roots.extend(env::split_paths(dirs).filter(|path| path.is_absolute())),
            None => roots.extend([
                PathBuf::from("/usr/local/share"),
                PathBuf::from("/usr/share"),
            ]),
        }
        if let Some(data_home) = data_home {
            roots.push(data_home.join("flatpak/exports/share"));
        }
        roots.push(PathBuf::from("/var/lib/flatpak/exports/share"));
        roots.push(PathBuf::from("/var/lib/snapd/desktop"));
        Self::from_data_dirs(roots)
    }

    pub(super) fn from_env() -> Self {
        Self::from_environment(
            env::var_os("XDG_DATA_HOME").as_deref(),
            env::var_os("XDG_DATA_DIRS").as_deref(),
            env::var_os("HOME").as_deref(),
        )
    }
}

#[cfg(test)]
mod tests;
