use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use models::*;

mod backend;
mod commands;
mod error;
mod models;

pub use error::{Error, Result};

/// Initializes the plugin.
pub fn init<R: Runtime>(locale: Option<String>) -> TauriPlugin<R> {
    Builder::new("i18n")
        .invoke_handler(tauri::generate_handler![
            commands::load_translations,
            commands::translate,
            commands::set_locale,
            commands::get_locale,
            commands::get_available_locales,
        ])
        .setup(|app, _api| {
            app.manage(PluginI18n::new(
                app.clone(),
                locale.unwrap_or("en".to_string()),
            ));

            Ok(())
        })
        .build()
}
