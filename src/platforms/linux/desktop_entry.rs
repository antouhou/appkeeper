use std::env;
use std::fs;
use std::path::Path;

use crate::app_entry::{AppEntry, AppStatus};

use super::desktop_entry_parser::DesktopEntry;
use super::paths::{command_exists, resolve_icon_paths};

/// Reads desktop file at a given path and tries to convert it into an `AppEntry`
pub(super) fn desktop_file_to_app(path: &Path) -> Option<AppEntry> {
    let contents = fs::read_to_string(path).ok()?;
    let entry = DesktopEntry::parse(&contents)?;

    if entry.raw("Type").is_none_or(|value| value != "Application") {
        return None;
    }

    if entry.boolean("Hidden")
        || entry.boolean("NoDisplay")
        || !is_visible_on_current_desktop(&entry)
    {
        return None;
    }

    if let Some(try_exec) = entry.string("TryExec")
        && !command_exists(&try_exec)
    {
        return None;
    }

    let name = entry.string("Name")?;

    Some(AppEntry {
        id: app_id_from_desktop_file(path)?,
        name,
        version: None,
        description: entry.string("Comment"),
        publisher: None,
        install_location: path.parent().map(Path::to_path_buf),
        icons: entry
            .string("Icon")
            .map(|icon| resolve_icon_paths(&icon))
            .unwrap_or_default(),
        categories: entry
            .string_list("Categories")
            .map(|categories| {
                categories
                    .into_iter()
                    .filter(|category| !category.is_empty())
                    .collect()
            })
            .unwrap_or_default(),
        status: AppStatus::default(),
        launch: entry.launch_command(path, entry.boolean("Terminal")),
    })
}

fn is_visible_on_current_desktop(entry: &DesktopEntry) -> bool {
    let current_desktops = current_desktops();

    if let Some(only_show_in) = entry.string_list("OnlyShowIn") {
        if current_desktops.is_empty() {
            return false;
        }

        if !only_show_in
            .iter()
            .any(|desktop| current_desktops.iter().any(|current| current == desktop))
        {
            return false;
        }
    }

    if let Some(not_show_in) = entry.string_list("NotShowIn")
        && not_show_in
            .iter()
            .any(|desktop| current_desktops.iter().any(|current| current == desktop))
    {
        return false;
    }

    true
}

fn current_desktops() -> Vec<String> {
    env::var("XDG_CURRENT_DESKTOP")
        .ok()
        .map(|desktops| {
            desktops
                .split(':')
                .filter(|desktop| !desktop.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn app_id_from_desktop_file(path: &Path) -> Option<String> {
    let file_name = path.file_name()?.to_string_lossy();
    Some(
        file_name
            .strip_suffix(".desktop")
            .unwrap_or(&file_name)
            .to_string(),
    )
}
