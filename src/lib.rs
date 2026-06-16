use crate::app_provider::AppProvider;

mod app_entry;
mod app_provider;
mod platforms;

pub fn get_provider() -> impl AppProvider {
    #[cfg(target_os = "linux")]
    {
        platforms::linux::LinuxAppProvider::new()
    }

    #[cfg(not(target_os = "linux"))]
    {
        platforms::mock::MockProvider
    }
}
