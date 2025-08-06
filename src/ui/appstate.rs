use nucleo::{Matcher, Utf32Str};
use ratatui::style::{Color, Style};
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

impl<'a> AppState<'a> {
    pub fn new(all_files: &[PathBuf], matcher: &mut nucleo::Matcher) -> Self {
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
            self.selected_path = Some(path.clone());

            let mut lines_for_no_preview: Vec<Line<'static>> = Vec::new();
            let no_preview_text = "No Preview available";
            let prev_width = self.curr_preview_width;
            let prev_height = self.curr_preview_height;

            let mut first_line_spans = Vec::new();
            first_line_spans.push(Span::styled(
                no_preview_text.to_string(),
                Style::default().fg(Color::DarkGray),
            ));

            let current_len = no_preview_text.len();
            if current_len < prev_width as usize {
                first_line_spans.push(Span::styled(
                    " ".repeat(prev_width as usize - current_len),
                    Style::default(),
                ));
            }
            lines_for_no_preview.push(Line::from(first_line_spans));

            while (lines_for_no_preview.len() as u16) < prev_height {
                lines_for_no_preview.push(Line::from(Span::styled(
                    " ".repeat(prev_width as usize),
                    Style::default(),
                )));
            }
            if !self.preview_cache.contains_key(path) {
                if let Ok(content) = std::fs::read_to_string(path) {
                    let highlighted = highlight_contents(
                        path,
                        &content,
                        self.curr_preview_height,
                        self.curr_preview_width,
                    );
                    self.preview_cache.insert(path.clone(), highlighted);
                } else {
                    self.preview_cache
                        .insert(path.clone(), Text::from(lines_for_no_preview));
                }
            }
        } else {
            self.selected_path = None;
            self.preview_cache.clear();
        }
    }
}
