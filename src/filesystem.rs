use crate::Result;
use anyhow::bail;
use std::path::PathBuf;

pub fn collect_files(starting_path: &str, hidden: bool) -> Result<Vec<PathBuf>> {
    let entry = std::fs::read_dir(starting_path)?;
    let mut file_vec = Vec::new();

    for entry_res in entry {
        let entry = entry_res?;
        let path = entry.path();

        if path.is_file() || (!is_hidden(&path) && hidden) {
            file_vec.push(path);
        }
    }

    Ok(file_vec)
}

fn is_hidden(path: &PathBuf) -> bool {
    path.file_name()
        .map(|name| name.to_string_lossy().starts_with("."))
        .unwrap_or(false)
}
