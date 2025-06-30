use crate::Result;
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::path::PathBuf;

pub fn collect_files(starting_path: &str, toggle_hidden: bool) -> Result<Vec<PathBuf>> {
    // let mut file_vec = Vec::new();
    // for entry_res in WalkBuilder::new(starting_path)
    //     .hidden(toggle_hidden)
    //     .build()
    // {
    //     let entry = entry_res?;
    //     let path = entry.path().to_path_buf();
    //     // if you want to use walkdir uncomment this
    //     // if is_hidden(&path) && !show_hidden {
    //     //     continue;
    //     // }
    //
    //     file_vec.push(path);
    // }
    //

    let file_vec = WalkBuilder::new(starting_path)
        .hidden(toggle_hidden)
        .build()
        .par_bridge()
        .map(|entry| entry.unwrap().path().to_path_buf())
        .collect();

    Ok(file_vec)
}

/// Deprecated function, used in tandem with walkdir
fn is_hidden(path: &PathBuf) -> bool {
    path.file_name()
        .map(|name| name.to_string_lossy().starts_with("."))
        .unwrap_or(false)
}
