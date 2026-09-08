use super::super::desktop_entry;
use super::super::paths::SearchPaths;
use crate::AppId;
use crate::app_entry::AppEntry;
use crate::app_provider::AppProviderEvent;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

pub(super) type AppCatalog = HashMap<AppId, AppEntry>;

#[derive(Debug, thiserror::Error)]
#[error("cannot scan {path}: {source}")]
pub(super) struct ScanError {
    path: PathBuf,
    source: io::Error,
}

/// Compares two catalogs, returns the list of changes - additions, removals and changes
pub(super) fn changes(previous: &AppCatalog, next: &AppCatalog) -> Vec<AppProviderEvent> {
    let ids = previous.keys().chain(next.keys()).collect::<BTreeSet<_>>();
    ids.into_iter()
        .filter_map(|id| match (previous.get(id), next.get(id)) {
            (None, Some(entry)) => Some(AppProviderEvent::Added(entry.clone())),
            (Some(_), None) => Some(AppProviderEvent::Removed(id.clone())),
            (Some(before), Some(after)) if before != after => {
                Some(AppProviderEvent::EntryUpdated(after.clone()))
            }
            _ => None,
        })
        .collect()
}

/// Reads the directory, iterates over the files, collects the files with .desktop extension into
/// the passed candidates vec. Recursive: calls [`collect_desktop_files`] on directories, which in
/// turn call this function.
fn collect_directory(
    root: &Path,
    directory: &Path,
    ancestors: &mut HashSet<(u64, u64)>,
    candidates: &mut Vec<(String, PathBuf)>,
) -> Result<(), ScanError> {
    let entries = fs::read_dir(directory).map_err(|source| ScanError {
        path: directory.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| ScanError {
            path: directory.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|source| ScanError {
            path: path.clone(),
            source,
        })?;
        if file_type.is_dir() || (file_type.is_symlink() && path.is_dir()) {
            collect_desktop_files(root, &path, ancestors, candidates)?;
        } else if (file_type.is_file() || file_type.is_symlink())
            && path
                .extension()
                .is_some_and(|extension| extension == "desktop")
        {
            let Some(relative) = path.strip_prefix(root).ok().and_then(Path::to_str) else {
                tracing::warn!(path = ?path, "desktop file has no UTF-8 relative path");
                continue;
            };
            candidates.push((relative.to_string(), path));
        }
    }
    Ok(())
}

/// Recursively walks all directories and collects all found desktop files into the passed vec.
fn collect_desktop_files(
    root: &Path,
    directory: &Path,
    ancestors: &mut HashSet<(u64, u64)>,
    candidates: &mut Vec<(String, PathBuf)>,
) -> Result<(), ScanError> {
    let metadata = match fs::metadata(directory) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(ScanError {
                path: directory.to_path_buf(),
                source,
            });
        }
    };
    if !metadata.is_dir() {
        return Err(ScanError {
            path: directory.to_path_buf(),
            source: io::Error::from(io::ErrorKind::NotADirectory),
        });
    }
    let inode = (metadata.dev(), metadata.ino());
    if !ancestors.insert(inode) {
        return Ok(());
    }
    let result = collect_directory(root, directory, ancestors, candidates);
    ancestors.remove(&inode);
    result
}

/// Recursively walks all directories passed, collects all desktop files and parses them.
/// Select paths before parsing so hidden and invalid overrides still claim IDs.
pub(super) fn load_apps(paths: &SearchPaths) -> Result<AppCatalog, ScanError> {
    let mut selected = HashMap::new();
    for root in paths.application_dirs() {
        let mut candidates = Vec::new();
        collect_desktop_files(root, root, &mut HashSet::new(), &mut candidates)?;
        candidates.sort_by(|left, right| left.0.cmp(&right.0));
        for (relative, path) in candidates {
            let id = AppId::from_parts("freedesktop", relative.replace('/', "-"));
            selected.entry(id).or_insert(path);
        }
    }
    Ok(selected
        .into_iter()
        .filter_map(|(id, path)| {
            desktop_entry::desktop_file_to_app(&path, id.clone(), paths.data_dirs())
                .map(|app| (id, app))
        })
        .collect())
}
