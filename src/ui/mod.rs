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

        let max_visible = terminal.size()?.height.saturating_sub(6);

        if state.selected_idx < state.scroll_offset as usize {
            state.scroll_offset = state.selected_idx as u16;
        } else if state.selected_idx as u16 >= state.scroll_offset.saturating_add(max_visible) {
        }
        state.scroll_offset = (state.selected_idx as u16).saturating_sub(max_visible + 1);

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
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
