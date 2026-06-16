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
            app(MockApp {
                id: "notes",
                name: "Notes",
                version: "1.4.2",
                publisher: "Acme Labs",
                icon: "comment-alt.svg",
                categories: &["Productivity"],
                is_running: true,
                is_launching: false,
                args: &["--new-window"],
            }),
            app(MockApp {
                id: "calendar",
                name: "Calendar",
                version: "3.2.0",
                publisher: "Northstar",
                icon: "calendar-days.svg",
                categories: &["Productivity", "Office"],
                is_running: true,
                is_launching: false,
                args: &[],
            }),
            app(MockApp {
                id: "mail",
                name: "Mail",
                version: "2.8.1",
                publisher: "Acme Labs",
                icon: "envelope.svg",
                categories: &["Communication"],
                is_running: true,
                is_launching: false,
                args: &["--compose"],
            }),
            app(MockApp {
                id: "messages",
                name: "Messages",
                version: "5.1.0",
                publisher: "Signal Works",
                icon: "message.svg",
                categories: &["Communication"],
                is_running: true,
                is_launching: false,
                args: &[],
            }),
            app(MockApp {
                id: "browser",
                name: "Browser",
                version: "119.0.3",
                publisher: "Open Coast",
                icon: "compass.svg",
                categories: &["Internet"],
                is_running: true,
                is_launching: false,
                args: &["--profile", "default"],
            }),
            app(MockApp {
                id: "files",
                name: "Files",
                version: "44.1",
                publisher: "Northstar",
                icon: "folder-open.svg",
                categories: &["System", "Utilities"],
                is_running: true,
                is_launching: false,
                args: &[],
            }),
            app(MockApp {
                id: "photos",
                name: "Photos",
                version: "7.0.5",
                publisher: "Pixel Foundry",
                icon: "images.svg",
                categories: &["Graphics", "Photography"],
                is_running: true,
                is_launching: false,
                args: &[],
            }),
            app(MockApp {
                id: "camera",
                name: "Camera",
                version: "1.9.0",
                publisher: "Pixel Foundry",
                icon: "camera.svg",
                categories: &["Graphics", "Photography"],
                is_running: false,
                is_launching: false,
                args: &[],
            }),
            app(MockApp {
                id: "music",
                name: "Music",
                version: "6.3.2",
                publisher: "Waveform",
                icon: "file-audio.svg",
                categories: &["Audio", "Media"],
                is_running: true,
                is_launching: false,
                args: &[],
            }),
            app(MockApp {
                id: "videos",
                name: "Videos",
                version: "4.6.1",
                publisher: "Waveform",
                icon: "file-video.svg",
                categories: &["Video", "Media"],
                is_running: false,
                is_launching: true,
                args: &[],
            }),
            app(MockApp {
                id: "terminal",
                name: "Terminal",
                version: "0.18.0",
                publisher: "Core Tools",
                icon: "window-maximize.svg",
                categories: &["System", "Developer"],
                is_running: true,
                is_launching: false,
                args: &["--working-directory", "~"],
            }),
            app(MockApp {
                id: "settings",
                name: "Settings",
                version: "12.0.0",
                publisher: "Core Tools",
                icon: "gear-solid-full.svg",
                categories: &["System"],
                is_running: true,
                is_launching: false,
                args: &[],
            }),
            app(MockApp {
                id: "monitor",
                name: "System Monitor",
                version: "1.12.4",
                publisher: "Core Tools",
                icon: "display.svg",
                categories: &["System", "Utilities"],
                is_running: true,
                is_launching: false,
                args: &[],
            }),
            app(MockApp {
                id: "network",
                name: "Network",
                version: "2.1.5",
                publisher: "Core Tools",
                icon: "network-wired.svg",
                categories: &["System", "Network"],
                is_running: false,
                is_launching: false,
                args: &[],
            }),
            app(MockApp {
                id: "wifi",
                name: "Wi-Fi",
                version: "2.1.5",
                publisher: "Core Tools",
                icon: "wifi.svg",
                categories: &["System", "Network"],
                is_running: false,
                is_launching: false,
                args: &[],
            }),
            app(MockApp {
                id: "bluetooth",
                name: "Bluetooth",
                version: "2.0.7",
                publisher: "Core Tools",
                icon: "bluetooth-b.svg",
                categories: &["System"],
                is_running: false,
                is_launching: false,
                args: &[],
            }),
            app(MockApp {
                id: "printer",
                name: "Printer",
                version: "8.4.0",
                publisher: "Paper Trail",
                icon: "print.svg",
                categories: &["Office", "System"],
                is_running: false,
                is_launching: false,
                args: &[],
            }),
            app(MockApp {
                id: "contacts",
                name: "Contacts",
                version: "3.0.2",
                publisher: "Northstar",
                icon: "address-book.svg",
                categories: &["Office", "Communication"],
                is_running: false,
                is_launching: false,
                args: &[],
            }),
            app(MockApp {
                id: "maps",
                name: "Maps",
                version: "10.3.1",
                publisher: "Open Coast",
                icon: "map.svg",
                categories: &["Travel", "Navigation"],
                is_running: false,
                is_launching: false,
                args: &[],
            }),
            app(MockApp {
                id: "news",
                name: "News",
                version: "6.8.0",
                publisher: "Daily Bit",
                icon: "newspaper.svg",
                categories: &["News"],
                is_running: false,
                is_launching: false,
                args: &[],
            }),
            app(MockApp {
                id: "wallet",
                name: "Wallet",
                version: "1.5.9",
                publisher: "Northstar",
                icon: "credit-card.svg",
                categories: &["Finance"],
                is_running: false,
                is_launching: false,
                args: &[],
            }),
            app(MockApp {
                id: "games",
                name: "Games",
                version: "9.2.3",
                publisher: "Arcade Desk",
                icon: "gamepad.svg",
                categories: &["Game"],
                is_running: false,
                is_launching: false,
                args: &[],
            }),
            app(MockApp {
                id: "chess",
                name: "Chess",
                version: "2.7.4",
                publisher: "Arcade Desk",
                icon: "chess-knight.svg",
                categories: &["Game"],
                is_running: false,
                is_launching: false,
                args: &[],
            }),
            app(MockApp {
                id: "sports",
                name: "Sports",
                version: "4.1.2",
                publisher: "Daily Bit",
                icon: "futbol.svg",
                categories: &["Sports", "News"],
                is_running: false,
                is_launching: false,
                args: &[],
            }),
            app(MockApp {
                id: "tasks",
                name: "Tasks",
                version: "2.5.0",
                publisher: "Northstar",
                icon: "circle-check.svg",
                categories: &["Productivity"],
                is_running: false,
                is_launching: false,
                args: &[],
            }),
            app(MockApp {
                id: "clipboard",
                name: "Clipboard",
                version: "1.2.8",
                publisher: "Core Tools",
                icon: "clipboard.svg",
                categories: &["Utilities"],
                is_running: false,
                is_launching: false,
                args: &[],
            }),
            app(MockApp {
                id: "voice-recorder",
                name: "Voice Recorder",
                version: "3.3.3",
                publisher: "Waveform",
                icon: "microphone.svg",
                categories: &["Audio", "Utilities"],
                is_running: false,
                is_launching: false,
                args: &[],
            }),
            app(MockApp {
                id: "power",
                name: "Power",
                version: "1.0.0",
                publisher: "Core Tools",
                icon: "power.svg",
                categories: &["System"],
                is_running: false,
                is_launching: false,
                args: &[],
            }),
        ]
    }

    fn subscribe(&mut self, _cb: fn(AppProviderEvent)) {
        // Mock data is static for now.
    }
}

struct MockApp<'a> {
    id: &'a str,
    name: &'a str,
    version: &'a str,
    publisher: &'a str,
    icon: &'a str,
    categories: &'a [&'a str],
    is_running: bool,
    is_launching: bool,
    args: &'a [&'a str],
}

fn app(spec: MockApp<'_>) -> AppEntry {
    AppEntry {
        id: spec.id.to_string(),
        name: spec.name.to_string(),
        version: Some(spec.version.to_string()),
        description: Some(format!("{} mock application", spec.name)),
        publisher: Some(spec.publisher.to_string()),
        install_location: Some(mock_path(spec.id)),
        icon_path: Some(mock_icon(spec.icon)),
        categories: spec.categories.iter().map(ToString::to_string).collect(),
        status: AppStatus {
            is_running: spec.is_running,
            is_launching: spec.is_launching,
        },
        launch: Some(LaunchCommand {
            executable: mock_path(spec.id).join(spec.id),
            args: spec
                .args
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
