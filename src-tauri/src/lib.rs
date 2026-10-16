mod commands;
mod error;
mod imaging;
mod naming;
mod state;
mod video;

use state::{AppState, VideoState};
use tauri::{Manager, RunEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .manage(VideoState::default())
        .setup(|app| {
            video::workdir::init(app.handle());
            Ok(())
        })
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
            commands::video_intake::add_videos,
            commands::video_intake::remove_video,
            commands::video_intake::clear_videos,
            commands::video_compress::compress_videos,
            commands::video_compress::cancel_compress,
            commands::video_save::video_posters,
            commands::video_save::save_video,
            commands::video_save::save_videos_to_folder,
            commands::video_save::save_poster,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let RunEvent::Exit = event {
                app.state::<VideoState>().shutdown();
            }
        });
}
