mod commands;
mod error;
mod imaging;
mod naming;
mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::files::add_paths,
            commands::files::remove_item,
            commands::convert::convert,
            commands::convert::cancel_convert,
            commands::save::save_item,
            commands::save::save_zip,
            commands::save::save_to_folder,
            commands::crop::open_for_crop,
            commands::crop::render_crop,
            commands::crop::save_crop,
            commands::crop::send_crop_to_converter,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
