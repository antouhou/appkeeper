use std::process::Command;

use crate::app_entry::{AppEntry, LaunchArg, LaunchCommand};
use crate::app_launcher::{AppLauncher, LaunchError, LaunchOptions};
use crate::app_provider::{AppProvider, AppProviderEvent};

pub struct LinuxAppProvider;

impl AppProvider for LinuxAppProvider {
    fn list(&self) -> Vec<AppEntry> {
        vec![]
    }

    fn subscribe(&mut self, _cb: fn(AppProviderEvent)) {}

    fn rescan(&mut self) {}
}

impl LinuxAppProvider {
    pub fn new() -> LinuxAppProvider {
        LinuxAppProvider {}
    }
}

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
            LaunchArg::AppName => args.push(app.name.clone()),
            LaunchArg::Icon => {
                if let Some(icon_path) = &app.icon_path {
                    args.push("--icon".to_string());
                    args.push(icon_path.to_string_lossy().into_owned());
                }
            }
            LaunchArg::DesktopFile => {
                return Err(LaunchError::UnsupportedArgument("desktop file"));
            }
        }
    }

    Ok(args)
}
