use std::env;
use std::path::Path;

use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
pub enum OperatingSystem {
    Windows,
    Linux,
    MacOS,
    Unknown,
}

#[tauri::command]
pub fn get_current_os() -> OperatingSystem {
    match env::consts::OS {
        "windows" => OperatingSystem::Windows,
        "linux" => OperatingSystem::Linux,
        "macos" => OperatingSystem::MacOS,
        _ => OperatingSystem::Unknown,
    }
}

pub fn get_watch_path() -> &'static Path {
    match env::consts::OS {
        "windows" => Path::new("C:\\"),
        "linux" => Path::new("/"),
        "macos" => Path::new("/Users/yangzichun/Downloads"),
        _ => Path::new("/"),
    }
}
