use crossterm::{
    event::{self, Event},
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use nucleo::{Matcher, Utf32Str};
use ratatui::{Terminal, backend::CrosstermBackend, text::Text};
use std::{error::Error, io, path::PathBuf};

// Bring in our new modules
pub mod appstate;
pub mod event_handler;
pub mod renderer;

// Bring in types from our sub-modules
use appstate::{AppState, Focus};
use event_handler::AppAction; // Bring in the enum from event_handler

// Import highlight from the crate root
use crate::highlight::highlight_contents;

pub fn run_app(
    all_files: &Vec<PathBuf>,
    matcher: &mut nucleo::Matcher,
) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Clear(ClearType::All))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut buf = Vec::new();

    let mut state = AppState {
        query: String::new(),
        filtered_files: update_filtered_files(
            Utf32Str::new("", &mut buf), // empty query
            all_files,
            matcher,
        ),

        focus: Focus::SearchBar,
        scroll_offset: 0,
        selected_idx: 0,
        preview_cache: HashMap::new(),
        selected_path: None,
        curr_preview_height: 0,
        curr_preview_width: 0,
    };

    loop {
        buf.clear();

        terminal.draw(|f| {
            f.render_widget(RatatuiClear, f.area());

            let size = f.area();
            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(1)])
                .split(size);

            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(vertical_chunks[1]);

            // Inspired from television's solution (https://github.com/alexpasmantier/television)
            let search_chunk = vertical_chunks[0];
            let content_chunk = horizontal_chunks[0];
            let preview_chunk = horizontal_chunks[1];

            let preview_block_for_calc = Block::default().borders(Borders::ALL);
            let inner_preview_area = preview_block_for_calc.inner(preview_chunk);

            state.curr_preview_width = inner_preview_area.width;
            state.curr_preview_height = inner_preview_area.height;
            draw_search_bar(
                &state,
                search_chunk,
                f,
                matches!(state.focus, Focus::SearchBar),
            );
            draw_content_box(
                &state,
                content_chunk,
                f,
                matches!(state.focus, Focus::Results),
            );

            draw_file_preview(preview_chunk, f, &state);
        })?;

        let max_visible = terminal.size()?.height.saturating_sub(6);

        if state.selected_idx < state.scroll_offset as usize {
            state.scroll_offset = state.selected_idx as u16;
        } else if state.selected_idx as u16 >= state.scroll_offset + max_visible {
            state.scroll_offset = state.selected_idx as u16 - max_visible + 1
        }

        let prev_query = state.query.clone();
        let prev_selected = state.selected_idx;
        if let Event::Key(key) = event::read()? {
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
                    KeyCode::Esc => break,
                    KeyCode::Enter => state.focus = Focus::Results,
                    _ => {}
                },
                Focus::Results => match key.code {
                    KeyCode::Up => {
                        if state.selected_idx > 0 {
                            state.selected_idx -= 1;
                        } else {
                            state.selected_idx = state.filtered_files.len() - 1
                        }
                    }
                    KeyCode::Down => {
                        if state.selected_idx + 1 < state.filtered_files.len() {
                            state.selected_idx += 1;
                        } else {
                            state.selected_idx = 0
                        }
                    }
                    KeyCode::Tab => state.focus = Focus::SearchBar,
                    KeyCode::Esc => break,
                    KeyCode::Enter => {
                        if let Some((edit_path, _, _)) =
                            state.filtered_files.get(state.selected_idx)
                        {
                            edit::edit_file(edit_path)?;
                            break;
                        }
                    }
                    _ => {}
                },
            }
        }

        let query_lower = &state.query.to_lowercase();
        let query_utf32 = Utf32Str::new(query_lower, &mut buf);

        state.filtered_files = update_filtered_files(query_utf32, all_files, matcher);

        // Update filtered files if query changed
        if state.query != prev_query {
            let query_lower = &state.query.to_lowercase();
            let query_utf32 = Utf32Str::new(query_lower, &mut buf);
            state.filtered_files = update_filtered_files(query_utf32, all_files, matcher);

            // Reset selection if query changed
            state.selected_idx = 0;
            state.scroll_offset = 0;
        }

        // Update preview if selection changed or query changed
        if state.selected_idx != prev_selected || state.query != prev_query {
            update_preview(&mut state);
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn update_preview(app_state: &mut AppState) {
    if let Some((path, _, _)) = app_state.filtered_files.get(app_state.selected_idx) {
        if app_state.selected_path.as_ref() != Some(path)
            || !app_state.preview_cache.contains_key(path)
        {
            app_state.selected_path = Some(path.clone());
        }
        app_state.selected_path = Some(path.clone());
        if !app_state.preview_cache.contains_key(path) {
            if let Ok(content) = std::fs::read_to_string(path) {
                let highlighted = highlight_contents(
                    path,
                    &content,
                    app_state.curr_preview_height,
                    app_state.curr_preview_width,
                );
                app_state.preview_cache.insert(path.clone(), highlighted);
            } else {
                app_state
                    .preview_cache
                    .insert(path.clone(), Text::from("No Preview available"));
            }
        }
    } else {
        app_state.selected_path = None;
        app_state.preview_cache.clear();
    }
}
