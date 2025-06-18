use crate::Result;
use anyhow::bail;
use std::fs;
use std::path::PathBuf;

pub fn collect_files(starting_path: &str, show_hidden: bool) -> Result<Vec<PathBuf>> {
    let dir = if starting_path == "~" {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
    } else {
        PathBuf::from(starting_path)
    };

    let entry = fs::read_dir(dir)?;
    let mut file_vec = Vec::new();

    for entry_res in entry {
        let entry = entry_res?;
        let path = entry.path();
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
