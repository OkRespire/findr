use nucleo::{Matcher, Utf32Str};
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::highlight::highlight_contents;

pub enum Focus {
    SearchBar,
    Results,
}
pub struct AppState<'a> {
    pub query: String,
    pub filtered_files: Vec<(PathBuf, String, Vec<u32>)>,
    pub focus: Focus,
    pub selected_idx: usize,
    pub scroll_offset: u16,
    pub selected_path: Option<PathBuf>,
    pub preview_cache: HashMap<PathBuf, Text<'a>>,
    pub curr_preview_height: u16,
    pub curr_preview_width: u16,
}

<<<<<<< HEAD
impl AppState {
    pub fn new(all_files: &[PathBuf], matcher: &mut nucleo::Matcher) -> Self {
=======
impl<'a> AppState<'a> {
    pub fn new(all_files: &Vec<PathBuf>, matcher: &mut nucleo::Matcher) -> Self {
>>>>>>> refs/remotes/origin/main
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
        buf.clear();

        state
    }

    /// Updates files based on the query inputted
    pub fn update_filtered_files(
        &mut self,
        query_utf32: Utf32Str,
        contents: &[PathBuf],
        matcher: &mut Matcher,
    ) {
        self.filtered_files.clear();
        let mut scored_files = contents
            .iter()
            .filter_map(|path| {
                let name = path.file_name()?.to_string_lossy().to_lowercase();
                let mut name_buf = Vec::new();
                let name_utf32 = Utf32Str::new(&name, &mut name_buf);

                let mut match_buf = Vec::new();
                let files = matcher
                    .fuzzy_indices(name_utf32, query_utf32, &mut match_buf)
                    .map(|score| (score, path.clone(), name, match_buf.clone()));
                name_buf.clear();
                match_buf.clear();

                files
            })
            .collect::<Vec<_>>();

        // Sort by descending score
        scored_files.sort_by(|a, b| b.0.cmp(&a.0));
        self.filtered_files = scored_files
            .into_iter()
            .map(|(_, path, n, i)| (path, n, i))
            .collect::<Vec<(PathBuf, String, Vec<u32>)>>()
    }

    pub fn update_preview(&mut self) {
        if let Some((path, _, _)) = self.filtered_files.get(self.selected_idx) {
            if self.selected_path.as_ref() != Some(path) || !self.preview_cache.contains_key(path) {
                self.selected_path = Some(path.clone());
            }

            let cache_key = format!(
                "{}:{}x{}",
                path.to_string_lossy(),
                self.curr_preview_width,
                self.curr_preview_height
            );
            // if !self.preview_cache.contains_key(path) {
            if let Ok(content) = std::fs::read_to_string(path) {
                let highlighted = highlight_contents(path, &content);
                self.preview_cache.insert(path.clone(), highlighted);
            } else {
                let mut lines = Vec::new();
                lines.push(Line::from(Span::styled(
                    "No Preview available",
                    Style::default().fg(ratatui::prelude::Color::DarkGray),
                )));
                while (lines.len() as u16) < self.curr_preview_height {
                    lines.push(Line::from(Span::styled(
                        " ".repeat(self.curr_preview_width as usize), // Pad to full width
                        Style::default(),                             // Transparent background
                    )));
                }
                self.preview_cache.insert(path.clone(), Text::from(lines));
            }
            // }
        } else {
            self.selected_path = None;
            self.preview_cache.clear();
        }
    }
}
