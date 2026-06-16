use crate::app_launcher::AppLauncher;
use crate::app_provider::AppProvider;

pub mod app_entry;
pub mod app_launcher;
pub mod app_provider;
mod platforms;

pub fn app_provider() -> impl AppProvider {
    #[cfg(target_os = "linux")]
    {
        platforms::linux::LinuxAppProvider::new()
    }

    #[cfg(not(target_os = "linux"))]
    {
        platforms::mock::MockProvider
    }
}

pub fn app_launcher() -> impl AppLauncher {
    #[cfg(target_os = "linux")]
    {
        platforms::linux::LinuxAppLauncher::new()
    }

    #[cfg(not(target_os = "linux"))]
    {
        app_launcher::MockLauncher::default()
    }
}
