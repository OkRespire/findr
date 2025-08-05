use crossterm::{
    event::{self},
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
};
use std::{error::Error, io, path::PathBuf};

// Bring in our new modules
pub mod appstate;
pub mod event_handler;
pub mod renderer;

// Bring in types from our sub-modules
use appstate::AppState;
use event_handler::AppAction; // Bring in the enum from event_handler

// Import highlight from the crate root
use crate::highlight::highlight_contents;

pub fn run_app(all_files: &[PathBuf], matcher: &mut nucleo::Matcher) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Clear(ClearType::All))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut buf = Vec::new();
    terminal.clear()?;

    let mut state = AppState::new(all_files, matcher);

    loop {
        buf.clear();
        let size = terminal.get_frame().area();
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(size);

        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(vertical_chunks[1]);
        let preview_chunk = horizontal_chunks[1];

        let preview_block_for_calc = Block::default().borders(Borders::ALL);
        let inner_preview_area = preview_block_for_calc.inner(preview_chunk);

        state.curr_preview_width = inner_preview_area.width;
        state.curr_preview_height = inner_preview_area.height;

        terminal.draw(|f| {
            renderer::draw_ui(f, &mut state);
        })?;

        let max_visible = terminal.size()?.height.saturating_sub(6); // 6 accounts for the borders
        // and other widgets

        let event = event::read()?;
        match event_handler::handle_events(event, all_files, matcher, &mut buf, &mut state)? {
            AppAction::Quit => break,
            AppAction::Continue => (),
            AppAction::EditFile(path) => {
                disable_raw_mode()?;
                execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                event_handler::edit_file(path)?;
                enable_raw_mode()?;
                execute!(
                    terminal.backend_mut(),
                    EnterAlternateScreen,
                    Clear(ClearType::All)
                )?;
            }
        }

        if state.selected_idx < state.scroll_offset as usize {
            state.scroll_offset = (state.selected_idx as u16).saturating_sub(max_visible - 1);
        } else if state.selected_idx as u16 >= state.scroll_offset.saturating_add(max_visible) {
            state.scroll_offset = (state.selected_idx as u16).saturating_sub(max_visible);
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
<<<<<<< HEAD

fn update_preview(app_state: &mut AppState) {
    if let Some((path, _, _)) = app_state.filtered_files.get(app_state.selected_idx) {
        if app_state.selected_path.as_ref() != Some(path)
            || !app_state.preview_cache.contains_key(path)
        {
            app_state.selected_path = Some(path.clone());
        }
        app_state.selected_path = Some(path.clone());

        let mut lines_for_no_preview: Vec<Line<'static>> = Vec::new();
        let no_preview_text = "No Preview available";
        let prev_width = app_state.curr_preview_width;
        let prev_height = app_state.curr_preview_height;

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
                    .insert(path.clone(), Text::from(lines_for_no_preview));
            }
        }
    } else {
        app_state.selected_path = None;
        app_state.preview_cache.clear();
    }
}
=======
>>>>>>> refs/remotes/origin/main
