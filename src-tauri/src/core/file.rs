use std::sync::{Mutex, RwLock};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread;

use lazy_static::lazy_static;
use serde::Serialize;

use crate::core::system::{get_current_os, OperatingSystem};

lazy_static! {
    static ref ROOT_NODE: Arc<RwLock<FileNode>> = Arc::new(RwLock::new(FileNode::empty()));
    static ref SCANNING: AtomicBool = AtomicBool::new(false);
    static ref WATCH_PATH: Mutex<Option<String>> = Mutex::new(None);
    static ref SHOULD_STOP: AtomicBool = AtomicBool::new(false);
}

#[derive(Debug, Clone)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub value: u64,
    pub children: Vec<Arc<RwLock<FileNode>>>,
}

#[derive(Debug, Serialize)]
pub struct FileNodeForJs {
    pub name: String,
    pub path: String,
    pub value: u64,
    pub children: Vec<FileNodeForJs>,
}

impl From<FileNode> for FileNodeForJs {
    fn from(node: FileNode) -> Self {
        fn convert(node: FileNode) -> FileNodeForJs {
            FileNodeForJs {
                name: node.name,
                path: node.path,
                value: node.value,
                children: node.children.into_iter().map(|node| {
                    let node = node.read().unwrap();
                    convert(node.clone())
                }).collect(),
            }
        }
        convert(node)
    }
}

impl FileNode {
    fn empty() -> Self {
        Self {
            name: "".to_string(),
            path: "".to_string(),
            value: 0,
            children: vec![],
        }
    }
}

fn explore_directory(dir: &str, current_node: Arc<RwLock<FileNode>>, depth: usize) -> u64 {
    if SHOULD_STOP.load(Ordering::SeqCst) {
        return 0;
    }
    if dir.starts_with("/proc") && get_current_os() == OperatingSystem::Linux {
        let proc_path = std::path::Path::new("/proc").canonicalize();

        if let Ok(proc_path) = proc_path {
            if let Ok(dir_path) = std::path::Path::new(dir).canonicalize() {
                if dir_path.starts_with(proc_path) {
                    return 0;
                }
            }
        }
    }

    let mut total_size = 0u64;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_symlink() {
                    continue;
                }
                if path.is_dir() {
                    if depth <= 5 {
                        let child_node = Arc::new(RwLock::new(FileNode {
                            name: path.file_name().unwrap().to_str().unwrap().to_string(),
                            path: path.to_str().unwrap().to_string(),
                            value: 0,
                            children: vec![],
                        }));
                        current_node.write().unwrap().children.push(child_node);
                        let child_size = explore_directory(path.to_str().unwrap(), Arc::clone(&current_node.read().unwrap().children.last().unwrap()), depth + 1);
                        current_node.write().unwrap().value += child_size;
                        total_size += child_size;
                    } else {
                        let child_size = explore_directory(path.to_str().unwrap(), Arc::new(RwLock::new(FileNode::empty())), depth + 1);
                        current_node.write().unwrap().value += child_size;
                        total_size += child_size;
                    }
                } else if let Ok(metadata) = std::fs::metadata(&path) {
                    let file_size = metadata.len();
                    current_node.write().unwrap().value += file_size;
                    total_size += file_size;
                    if file_size > 5 * 1024 * 1024 && depth <= 5 {
                        let child_node = FileNode {
                            name: path.file_name().unwrap().to_str().unwrap().to_string(),
                            path: path.to_str().unwrap().to_string(),
                            value: file_size,
                            children: vec![],
                        };
                        current_node.write().unwrap().children.push(Arc::new(RwLock::new(child_node)));
                    }
                }
            }
        }
    }
    total_size
}

#[tauri::command]
pub async fn start_scan_folder(path: String) -> Result<(), String> {
    stop_scan_folder_and_clear().await;
    *WATCH_PATH.lock().unwrap() = Some(path.clone());
    SHOULD_STOP.store(false, Ordering::SeqCst);
    thread::spawn(move || {
        SCANNING.store(true, Ordering::SeqCst);
        let root_node = Arc::clone(&ROOT_NODE);
        let _node = explore_directory(path.as_str(), root_node, 0);
        SCANNING.store(false, Ordering::SeqCst);
    });
    Ok(())
}

#[tauri::command]
pub fn get_folder_info() -> Vec<FileNodeForJs> {
    let root_node = Arc::clone(&ROOT_NODE);
    let root_node = root_node.read().unwrap().clone();
    FileNodeForJs::from(root_node).children
}

#[tauri::command]
pub fn is_scanning() -> bool {
    SCANNING.load(Ordering::SeqCst)
}

#[tauri::command]
pub fn get_recommend_folders(current_path: String) -> Result<Vec<String>, String> {
    let path = std::path::Path::new(&current_path);
    let dir_to_read = if path.exists() {
        path.to_path_buf()
    } else {
        if let Some(parent) = path.parent() {
            parent.to_path_buf()
        } else {
            path.to_path_buf()
        }
    };
    let dir_to_read = dir_to_read.canonicalize().map_err(|e| format!("Error canonicalizing path: {}", e))?;
    if !dir_to_read.exists() {
        return Err(format!("Path does not exist: {}", dir_to_read.to_str().unwrap()));
    }

    let entries = std::fs::read_dir(&dir_to_read).or_else(|_| {
        std::fs::read_dir(dir_to_read.parent().unwrap_or(&dir_to_read))
    }).map_err(|e| format!("Error reading directory: {}", e))?;

    let lowercase_current_path = current_path.to_lowercase();
    let result: Vec<String> = entries
        .filter_map(|entry| {
            entry.ok().and_then(|entry| {
                let entry_path = entry.path();
                let path_str = entry_path.to_str()?;
                if entry_path.is_dir() && path_str.to_lowercase().starts_with(&lowercase_current_path) {
                    Some(path_str.to_string())
                } else {
                    None
                }
            })
        })
        .collect();

    Ok(result)
}

#[tauri::command]
pub async fn stop_scan_folder_and_clear() {
    SHOULD_STOP.store(true, Ordering::SeqCst);
    SCANNING.store(false, Ordering::SeqCst);
    *WATCH_PATH.lock().unwrap() = None;
    let root_node = Arc::clone(&ROOT_NODE);
    *root_node.write().unwrap() = FileNode::empty();
}

#[tauri::command]
pub async fn delete_path(path: String) {
    let path = std::path::Path::new(&path);
    if path.is_dir() {
        std::fs::remove_dir_all(path).unwrap();
    } else {
        std::fs::remove_file(path).unwrap();
    }
}