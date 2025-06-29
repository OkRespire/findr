use crate::Result;
use ignore::WalkBuilder;
use std::path::PathBuf;
pub fn collect_files(starting_path: &str, toggle_hidden: bool) -> Result<Vec<PathBuf>> {
    let mut file_vec = Vec::new();

    for entry_res in WalkBuilder::new(starting_path)
        .hidden(toggle_hidden)
        .build()
    {
        let entry = entry_res?;
        let path = entry.path().to_path_buf();
        // if is_hidden(&path) && !show_hidden {
        //     continue;
        // }

        file_vec.push(path);
    }

    Ok(file_vec)
}

fn is_hidden(path: &PathBuf) -> bool {
    path.file_name()
        .map(|name| name.to_string_lossy().starts_with("."))
        .unwrap_or(false)
}
