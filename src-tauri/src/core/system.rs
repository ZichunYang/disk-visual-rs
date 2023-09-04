use std::env;

#[derive(Debug, PartialEq)]
pub enum OperatingSystem {
    Windows,
    Linux,
    MacOS,
    Unknown,
}

pub fn get_current_os() -> OperatingSystem {
    match env::consts::OS {
        "windows" => OperatingSystem::Windows,
        "linux" => OperatingSystem::Linux,
        "macos" => OperatingSystem::MacOS,
        _ => OperatingSystem::Unknown,
    }
}
