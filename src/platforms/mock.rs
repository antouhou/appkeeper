#![allow(dead_code)]

use std::path::PathBuf;

use crate::app_entry::{AppEntry, AppStatus, LaunchArg, LaunchCommand};
use crate::app_provider::{AppProvider, AppProviderEvent};

pub struct MockProvider {}

impl MockProvider {
    pub fn new() -> Self {
        Self {}
    }
}

impl AppProvider for MockProvider {
    fn list(&self) -> Vec<AppEntry> {
        vec![
            app(
                "notes",
                "Notes",
                "1.4.2",
                "Acme Labs",
                "comment-alt.svg",
                &["Productivity"],
                true,
                false,
                &["--new-window"],
            ),
            app(
                "calendar",
                "Calendar",
                "3.2.0",
                "Northstar",
                "calendar-days.svg",
                &["Productivity", "Office"],
                true,
                false,
                &[],
            ),
            app(
                "mail",
                "Mail",
                "2.8.1",
                "Acme Labs",
                "envelope.svg",
                &["Communication"],
                true,
                false,
                &["--compose"],
            ),
            app(
                "messages",
                "Messages",
                "5.1.0",
                "Signal Works",
                "message.svg",
                &["Communication"],
                true,
                false,
                &[],
            ),
            app(
                "browser",
                "Browser",
                "119.0.3",
                "Open Coast",
                "compass.svg",
                &["Internet"],
                true,
                false,
                &["--profile", "default"],
            ),
            app(
                "files",
                "Files",
                "44.1",
                "Northstar",
                "folder-open.svg",
                &["System", "Utilities"],
                true,
                false,
                &[],
            ),
            app(
                "photos",
                "Photos",
                "7.0.5",
                "Pixel Foundry",
                "images.svg",
                &["Graphics", "Photography"],
                true,
                false,
                &[],
            ),
            app(
                "camera",
                "Camera",
                "1.9.0",
                "Pixel Foundry",
                "camera.svg",
                &["Graphics", "Photography"],
                false,
                false,
                &[],
            ),
            app(
                "music",
                "Music",
                "6.3.2",
                "Waveform",
                "file-audio.svg",
                &["Audio", "Media"],
                true,
                false,
                &[],
            ),
            app(
                "videos",
                "Videos",
                "4.6.1",
                "Waveform",
                "file-video.svg",
                &["Video", "Media"],
                false,
                true,
                &[],
            ),
            app(
                "terminal",
                "Terminal",
                "0.18.0",
                "Core Tools",
                "window-maximize.svg",
                &["System", "Developer"],
                true,
                false,
                &["--working-directory", "~"],
            ),
            app(
                "settings",
                "Settings",
                "12.0.0",
                "Core Tools",
                "gear-solid-full.svg",
                &["System"],
                true,
                false,
                &[],
            ),
            app(
                "monitor",
                "System Monitor",
                "1.12.4",
                "Core Tools",
                "display.svg",
                &["System", "Utilities"],
                true,
                false,
                &[],
            ),
            app(
                "network",
                "Network",
                "2.1.5",
                "Core Tools",
                "network-wired.svg",
                &["System", "Network"],
                false,
                false,
                &[],
            ),
            app(
                "wifi",
                "Wi-Fi",
                "2.1.5",
                "Core Tools",
                "wifi.svg",
                &["System", "Network"],
                false,
                false,
                &[],
            ),
            app(
                "bluetooth",
                "Bluetooth",
                "2.0.7",
                "Core Tools",
                "bluetooth-b.svg",
                &["System"],
                false,
                false,
                &[],
            ),
            app(
                "printer",
                "Printer",
                "8.4.0",
                "Paper Trail",
                "print.svg",
                &["Office", "System"],
                false,
                false,
                &[],
            ),
            app(
                "contacts",
                "Contacts",
                "3.0.2",
                "Northstar",
                "address-book.svg",
                &["Office", "Communication"],
                false,
                false,
                &[],
            ),
            app(
                "maps",
                "Maps",
                "10.3.1",
                "Open Coast",
                "map.svg",
                &["Travel", "Navigation"],
                false,
                false,
                &[],
            ),
            app(
                "news",
                "News",
                "6.8.0",
                "Daily Bit",
                "newspaper.svg",
                &["News"],
                false,
                false,
                &[],
            ),
            app(
                "wallet",
                "Wallet",
                "1.5.9",
                "Northstar",
                "credit-card.svg",
                &["Finance"],
                false,
                false,
                &[],
            ),
            app(
                "games",
                "Games",
                "9.2.3",
                "Arcade Desk",
                "gamepad.svg",
                &["Game"],
                false,
                false,
                &[],
            ),
            app(
                "chess",
                "Chess",
                "2.7.4",
                "Arcade Desk",
                "chess-knight.svg",
                &["Game"],
                false,
                false,
                &[],
            ),
            app(
                "sports",
                "Sports",
                "4.1.2",
                "Daily Bit",
                "futbol.svg",
                &["Sports", "News"],
                false,
                false,
                &[],
            ),
            app(
                "tasks",
                "Tasks",
                "2.5.0",
                "Northstar",
                "circle-check.svg",
                &["Productivity"],
                false,
                false,
                &[],
            ),
            app(
                "clipboard",
                "Clipboard",
                "1.2.8",
                "Core Tools",
                "clipboard.svg",
                &["Utilities"],
                false,
                false,
                &[],
            ),
            app(
                "voice-recorder",
                "Voice Recorder",
                "3.3.3",
                "Waveform",
                "microphone.svg",
                &["Audio", "Utilities"],
                false,
                false,
                &[],
            ),
            app(
                "power",
                "Power",
                "1.0.0",
                "Core Tools",
                "power.svg",
                &["System"],
                false,
                false,
                &[],
            ),
        ]
    }

    fn subscribe(&mut self, _cb: fn(AppProviderEvent)) {
        // Mock data is static for now.
    }

    fn rescan(&mut self) {}
}

fn app(
    id: &str,
    name: &str,
    version: &str,
    publisher: &str,
    icon: &str,
    categories: &[&str],
    is_running: bool,
    is_launching: bool,
    args: &[&str],
) -> AppEntry {
    AppEntry {
        id: id.to_string(),
        name: name.to_string(),
        version: Some(version.to_string()),
        description: Some(format!("{name} mock application")),
        publisher: Some(publisher.to_string()),
        install_location: Some(mock_path(id)),
        icon_path: Some(mock_icon(icon)),
        categories: categories
            .iter()
            .map(std::string::ToString::to_string)
            .collect(),
        status: AppStatus {
            is_running,
            is_launching,
        },
        launch: Some(LaunchCommand {
            executable: mock_path(id).join(id),
            args: args
                .iter()
                .map(|arg| LaunchArg::Literal((*arg).to_string()))
                .collect(),
            requires_terminal: false,
        }),
    }
}

fn mock_icon(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("mock_icons")
        .join(file_name)
}

fn mock_path(id: &str) -> PathBuf {
    PathBuf::from("/mock/apps").join(id)
}
