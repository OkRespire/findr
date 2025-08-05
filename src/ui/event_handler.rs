use crate::ui::appstate::{AppState, Focus};
use crossterm::event::{Event, KeyCode};
use nucleo::{Matcher, Utf32Str};
use std::error::Error;

pub use edit::edit_file;

pub enum AppAction {
    Quit,
    Continue,
    EditFile(std::path::PathBuf),
}

pub fn handle_events(
    event: Event,
    all_files: &[std::path::PathBuf],
    matcher: &mut Matcher,
    buf: &mut Vec<char>,
    state: &mut AppState,
) -> Result<AppAction, Box<dyn Error>> {
    let prev_query = state.query.clone();
    let prev_selected = state.selected_idx;
    if let Event::Key(key) = event {
        match state.focus {
            Focus::SearchBar => match key.code {
                KeyCode::Char(c) => {
                    state.query.push(c);
                }
                KeyCode::Backspace => {
                    state.query.pop();
                }
                KeyCode::Tab => {
                    state.focus = Focus::Results;
                }
                KeyCode::Esc => return Ok(AppAction::Quit),
                KeyCode::Enter => state.focus = Focus::Results,
                _ => {}
            },
            Focus::Results => match key.code {
                KeyCode::Up => {
                    if state.selected_idx > 0 {
                        state.selected_idx -= 1;
                    } else {
                        state.selected_idx = state.filtered_files.len() - 1;
                    }
                }
                KeyCode::Down => {
                    if state.selected_idx + 1 < state.filtered_files.len() {
                        state.selected_idx += 1;
                    } else {
                        state.selected_idx = 0;
                    }
                }
                KeyCode::Tab => state.focus = Focus::SearchBar,
                KeyCode::Esc => return Ok(AppAction::Quit),
                KeyCode::Enter => {
                    if let Some((edit_path, _, _)) = state.filtered_files.get(state.selected_idx) {
                        return Ok(AppAction::EditFile(edit_path.clone()));
                    }
                }
                _ => {}
            },
        }
    }

    buf.clear();
    let query_lower = &state.query.to_lowercase();
    let query_utf32 = Utf32Str::new(query_lower, buf);

    state.update_filtered_files(query_utf32, all_files, matcher);

    // Update filtered files if query changed
    if state.query != prev_query {
        if state.query.is_empty() {
            *matcher = nucleo::Matcher::default(); // reset matcher for empty queries (thanks
            // claude)
        }
        buf.clear();
        let query_lower = &state.query.to_lowercase();
        let query_utf32 = Utf32Str::new(query_lower, buf);
        state.update_filtered_files(query_utf32, all_files, matcher);

        // Reset selection if query changed
        state.selected_idx = 0;
        state.scroll_offset = 0;
    }

    // Update preview if selection changed or query changed
    if state.selected_idx != prev_selected || state.query != prev_query {
        state.update_preview();
    }

    Ok(AppAction::Continue)
}
