use std::process::Command;

use crate::app_entry::{AppEntry, LaunchArg, LaunchArgPart, LaunchCommand};
use crate::app_launcher::{AppLauncher, LaunchError, LaunchOptions};

pub struct LinuxAppLauncher;

impl LinuxAppLauncher {
    pub fn new() -> Self {
        Self
    }
}

impl AppLauncher for LinuxAppLauncher {
    fn launch(&self, app: &AppEntry, options: LaunchOptions) -> Result<(), LaunchError> {
        let launch = app.launch.as_ref().ok_or(LaunchError::NotLaunchable)?;
        let args = resolve_launch_args(app, launch, options)?;

        let mut command = if launch.requires_terminal {
            let mut command = Command::new("x-terminal-emulator");
            command.arg("-e").arg(&launch.executable);
            command
        } else {
            Command::new(&launch.executable)
        };

        command.args(args);
        command.spawn()?;

        Ok(())
    }
}

fn resolve_launch_args(
    app: &AppEntry,
    launch: &LaunchCommand,
    options: LaunchOptions,
) -> Result<Vec<String>, LaunchError> {
    let mut args = Vec::new();

    for arg in &launch.args {
        match arg {
            LaunchArg::Literal(value) => args.push(value.clone()),
            LaunchArg::File => {
                if let Some(file) = options.files.first() {
                    args.push(file.to_string_lossy().into_owned());
                }
            }
            LaunchArg::Files => {
                args.extend(
                    options
                        .files
                        .iter()
                        .map(|file| file.to_string_lossy().into_owned()),
                );
            }
            LaunchArg::Url => {
                if let Some(url) = options.urls.first() {
                    args.push(url.clone());
                }
            }
            LaunchArg::Urls => args.extend(options.urls.iter().cloned()),
            LaunchArg::Template(parts) => {
                args.push(resolve_launch_arg_template(app, parts, &options))
            }
            LaunchArg::AppName => args.push(app.name.clone()),
            LaunchArg::Icon(icon) => {
                args.push("--icon".to_string());
                args.push(icon.clone());
            }
            LaunchArg::DesktopFile(path) => args.push(path.to_string_lossy().into_owned()),
        }
    }

    Ok(args)
}

fn resolve_launch_arg_template(
    app: &AppEntry,
    parts: &[LaunchArgPart],
    options: &LaunchOptions,
) -> String {
    let mut value = String::new();

    for part in parts {
        match part {
            LaunchArgPart::Literal(part) => value.push_str(part),
            LaunchArgPart::File => {
                if let Some(file) = options.files.first() {
                    value.push_str(&file.to_string_lossy());
                }
            }
            LaunchArgPart::Url => {
                if let Some(url) = options.urls.first() {
                    value.push_str(url);
                }
            }
            LaunchArgPart::AppName => value.push_str(&app.name),
            LaunchArgPart::DesktopFile(path) => value.push_str(&path.to_string_lossy()),
        }
    }

    value
}
