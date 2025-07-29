use nucleo::{Matcher, Utf32Str};
use ratatui::text::Text;
use std::collections::HashMap;
use std::path::PathBuf;

pub enum Focus {
    SearchBar,
    Results,
}
pub struct AppState {
    pub query: String,
    pub filtered_files: Vec<(PathBuf, String, Vec<u32>)>,
    pub focus: Focus,
    pub selected_idx: usize,
    pub scroll_offset: u16,
    pub selected_path: Option<PathBuf>,
    pub preview_cache: HashMap<PathBuf, Text<'static>>,
    pub curr_preview_height: u16,
    pub curr_preview_width: u16,
}

impl AppState {
    pub fn new(all_files: &Vec<PathBuf>, matcher: &mut nucleo::Matcher) -> Self {
        let mut buf = Vec::new(); // Local buffer for UTF32 conversion
        let mut state = AppState {
            query: String::new(),
            filtered_files: Vec::new(),
            focus: Focus::SearchBar,
            scroll_offset: 0,
            selected_idx: 0,
            preview_cache: HashMap::new(),
            selected_path: None,
            curr_preview_height: 0,
            curr_preview_width: 0,
        };

        state.update_filtered_files(nucleo::Utf32Str::new("", &mut buf), all_files, matcher);

        state
    }

    /// Updates files based on the query inputted
    pub fn update_filtered_files(
        &mut self,
        query_utf32: Utf32Str,
        contents: &[PathBuf],
        matcher: &mut Matcher,
    ) {
        let mut scored_files = contents
            .iter()
            .filter_map(|path| {
                let name = path.file_name()?.to_string_lossy().to_lowercase();
                let mut name_buf = Vec::new();
                let name_utf32 = Utf32Str::new(&name, &mut name_buf);

                let mut match_buf = Vec::new();
                matcher
                    .fuzzy_indices(name_utf32, query_utf32, &mut match_buf)
                    .map(|score| (score, path.clone(), name, match_buf.clone()))
            })
            .collect::<Vec<_>>();

        // Sort by descending score
        scored_files.sort_by(|a, b| b.0.cmp(&a.0));
        self.filtered_files = scored_files
            .into_iter()
            .map(|(_, path, n, i)| (path, n, i))
            .collect::<Vec<(PathBuf, String, Vec<u32>)>>()
    }
}
