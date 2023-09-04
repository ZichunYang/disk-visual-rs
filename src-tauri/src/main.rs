// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod core {
    pub mod file;
    pub mod system;
}

fn main() {
    // let mut watcher = notify::recommended_watcher(|res| match res {
    //     Ok(event) => println!("event: {:?}", event),
    //     Err(e) => println!("watch error: {:?}", e),
    // }).unwrap();
    // let path = core::system::get_watch_path();
    // watcher.watch(path, RecursiveMode::Recursive).unwrap();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            core::file::start_scan_folder,
            core::file::stop_scan_folder_and_clear,
            core::file::delete_path,
            core::file::is_scanning,
            core::file::get_folder_info,
            core::system::get_current_os,
            core::file::get_recommend_folders,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
