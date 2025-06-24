use crate::Result;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

pub fn collect_files(starting_path: &str, show_hidden: bool) -> Result<Vec<PathBuf>> {
    let dir = if starting_path == "~" {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
    } else {
        PathBuf::from(starting_path)
    };

    let mut file_vec = Vec::new();

    for entry_res in WalkDir::new(starting_path) {
        let entry = entry_res?;
        let path = entry.path().to_path_buf();
        if is_hidden(&path) && !show_hidden {
            continue;
        }

        file_vec.push(path);
    }

    Ok(file_vec)
}

fn is_hidden(path: &PathBuf) -> bool {
    path.file_name()
        .map(|name| name.to_string_lossy().starts_with("."))
        .unwrap_or(false)
}
