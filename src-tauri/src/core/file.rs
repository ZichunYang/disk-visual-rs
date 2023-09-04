use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::{cmp::Ordering, fmt::Display};

use crate::core::system::{self, OperatingSystem};

#[derive(Debug, Clone)]
struct FileNode {
    name: String,
    size: u64,
    children: BinaryHeap<Reverse<FileNode>>,
}

impl PartialEq for FileNode {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size
    }
}

impl Eq for FileNode {}

impl PartialOrd for FileNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FileNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.size.cmp(&other.size)
    }
}

fn explore_directory(dir: &str) -> FileNode {
    let mut total_size = 0;
    let mut children = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    let child_node = explore_directory(path.to_str().unwrap());
                    total_size += child_node.size;
                    children.push(Reverse(child_node));
                } else if let Ok(metadata) = std::fs::metadata(&path) {
                    let file_size = metadata.len();
                    total_size += file_size;
                    children.push(Reverse(FileNode {
                        name: path.to_str().unwrap().to_string(),
                        size: file_size,
                        children: BinaryHeap::new(),
                    }));
                }
            }
        }
    }

    FileNode {
        name: dir.to_string(),
        size: total_size,
        children: BinaryHeap::from(children),
    }
}

fn format_file_size(byte: u64) -> String {
    let operating_system = system::get_current_os();
    let base_unit: f64 = if operating_system == OperatingSystem::Windows {
        1024.0
    } else if operating_system == OperatingSystem::MacOS
        || operating_system == OperatingSystem::Linux
    {
        1000.0
    } else {
        1024.0 // default to 1024 for other OS
    };

    if byte < base_unit as u64 {
        return format!("{} B", byte);
    }
    let units = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    let mut size = byte as f64;
    let mut reduced_size = size;
    let mut unit_idx = 0;

    while size >= base_unit && unit_idx < units.len() - 1 {
        reduced_size = size;
        size /= base_unit;
        unit_idx += 1;
    }

    if size < 1.0 && unit_idx > 0 {
        unit_idx -= 1;
        size = reduced_size;
    }

    format!("{:.2} {}", size, units[unit_idx])
}

#[test]
fn test_explore_directory() {
    let dir = "/Users/yangzichun/Downloads";
    let file_node = explore_directory(dir);
    println!("Total size: {:?}", file_node);
}
