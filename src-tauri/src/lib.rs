pub mod commands;
pub mod helper;
pub mod hosts;
pub mod models;
pub mod repositories;
pub mod state;

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, WindowEvent,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let show = MenuItem::with_id(app, "show", "Abrir Local Domains", true, None::<&str>)?;

            let quit = MenuItem::with_id(app, "quit", "Salir", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&show, &quit])?;

            let app_data_dir = app.path().app_data_dir()?;

            let repository = repositories::DomainRepository::new(app_data_dir);

            let domains = repository
                .load()
                .map_err(|error| std::io::Error::other(error))?;

            app.manage(state::AppState::new(repository, domains));

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }

                    "quit" => {
                        app.exit(0);
                    }

                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::domains::get_domains,
            commands::domains::toggle_domain,
            commands::domains::create_domain,
            commands::domains::update_domain,
            commands::domains::delete_domain,
            commands::domains::enable_all,
            commands::domains::disable_all,
            commands::domains::toggle_group,
            commands::domains::export_domains,
            commands::domains::import_domains,
            commands::domains::flush_dns,
            commands::domains::helper_status,
            commands::domains::get_hosts_content,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
