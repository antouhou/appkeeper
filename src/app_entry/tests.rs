use super::*;
use std::path::Path;

#[test]
fn picks_best_icon_for_requested_size() {
    let app = AppEntry {
        id: AppId::from_parts("mock", "test"),
        name: "Test".to_string(),
        version: None,
        description: None,
        publisher: None,
        install_location: None,
        icons: vec![
            AppIcon {
                path: PathBuf::from("unknown.xpm"),
                size: AppIconSize::Unknown,
            },
            AppIcon {
                path: PathBuf::from("64.png"),
                size: AppIconSize::Pixels(64),
            },
            AppIcon {
                path: PathBuf::from("32.png"),
                size: AppIconSize::Pixels(32),
            },
            AppIcon {
                path: PathBuf::from("icon.svg"),
                size: AppIconSize::Scalable,
            },
        ],
        categories: Vec::new(),
        status: AppStatus::default(),
        launch_command: None,
    };

    assert_eq!(
        app.icon_for_size(32).map(|icon| icon.path.as_path()),
        Some(Path::new("32.png"))
    );
    assert_eq!(
        app.icon_for_size(48).map(|icon| icon.path.as_path()),
        Some(Path::new("icon.svg"))
    );
}
