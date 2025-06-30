use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use edit;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Color,
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};

use nucleo::{Matcher, Utf32Str};
use std::{collections::HashMap, error::Error, io, path::PathBuf};

use crate::highlight::highlight_contents;

struct AppState {
    query: String,
    filtered_files: Vec<PathBuf>,
    focus: Focus,
    selected_idx: usize,
    scroll_offset: u16,
    selected_path: Option<PathBuf>,
    preview_cache: HashMap<PathBuf, Text<'static>>,
}

enum Focus {
    SearchBar,
    Results,
}

pub fn run_app(
    all_files: &Vec<PathBuf>,
    matcher: &mut nucleo::Matcher,
) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut buf = Vec::new();

    let mut state = AppState {
        query: String::new(),
        filtered_files: all_files.clone(),
        focus: Focus::SearchBar,
        scroll_offset: 2,
        selected_idx: 0,
        preview_cache: HashMap::new(),
        selected_path: None,
    };

    loop {
        buf.clear();
        terminal.draw(|f| {
            let size = f.area();
            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(1)])
                .split(size);

            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(vertical_chunks[1]);
            draw_search_bar(
                &state.query,
                vertical_chunks[0],
                f,
                matches!(state.focus, Focus::SearchBar),
            );
            draw_content_box(
                &state.filtered_files,
                horizontal_chunks[0],
                f,
                matches!(state.focus, Focus::Results),
                state.selected_idx,
                state.scroll_offset,
            );

            draw_file_preview(
                horizontal_chunks[1],
                f,
                &state.preview_cache,
                &state.selected_path,
            );
            let max_visible = horizontal_chunks[0].height.saturating_sub(3);

            if state.selected_idx < state.scroll_offset as usize {
                state.scroll_offset = state.selected_idx as u16;
            } else if state.selected_idx as u16 >= state.scroll_offset + max_visible {
                state.scroll_offset = state.selected_idx as u16 - max_visible + 1
            }
        })?;

        if let Event::Key(key) = event::read()? {
            match state.focus {
                Focus::SearchBar => match key.code {
                    KeyCode::Char(c) => state.query.push(c),
                    KeyCode::Backspace => {
                        state.query.pop();
                    }
                    KeyCode::Tab => state.focus = Focus::Results,
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
                        if let Some(edit_path) = state.filtered_files.get(state.selected_idx) {
                            edit::edit_file(edit_path)?;
                            break;
                        }
                    }
                    _ => {}
                },
            }
        }

        let query_lower = &state.query.to_lowercase();
        let query_utf32 = Utf32Str::new(&query_lower, &mut buf);

        state.filtered_files = update_filtered_files(query_utf32, all_files, matcher);

        if let Some(path) = state.filtered_files.get(state.selected_idx) {
            state.selected_path = Some(path.clone());
            if !state.preview_cache.contains_key(path) {
                if let Ok(content) = std::fs::read_to_string(path) {
                    let highlighted = highlight_contents(path, &content);
                    state.preview_cache.insert(path.clone(), highlighted);
                } else {
                    state
                        .preview_cache
                        .insert(path.clone(), Text::from("No Preview available"));
                }
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

/// Uses nucleo to score files and ensures it is going in descending order, through the score
fn update_filtered_files(
    query_utf32: Utf32Str,
    contents: &[PathBuf],
    matcher: &mut Matcher,
) -> Vec<PathBuf> {
    let mut scored_files = contents
        .iter()
        .filter_map(|path| {
            let name = path.file_name()?.to_string_lossy().to_lowercase();
            let mut name_buf = Vec::new();
            let name_utf32 = Utf32Str::new(&name, &mut name_buf);

            matcher
                .fuzzy_match(name_utf32, query_utf32)
                .map(|score| (score, path.clone()))
        })
        .collect::<Vec<_>>();

    // Sort by descending score
    scored_files.sort_by(|a, b| b.0.cmp(&a.0));
    scored_files
        .into_iter()
        .map(|(_, p)| p)
        .collect::<Vec<PathBuf>>()
}

fn draw_content_box(
    contents: &[PathBuf],
    size: Rect,
    f: &mut Frame<'_>,
    focused: bool,
    s_idx: usize,
    scroll_offset: u16,
) {
    let line_of_content: Vec<Line> = contents
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let text = p.display().to_string();
            if i == s_idx {
                Line::from(Span::styled(
                    text,
                    Style::default().bg(Color::White).fg(Color::Black),
                ))
            } else {
                Line::from(Span::raw(text))
            }
        })
        .collect();

    let content_box = Paragraph::new(Text::from(line_of_content))
        .block(
            Block::default()
                .title("Files")
                .borders(Borders::ALL)
                .style(if focused {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                }),
        )
        .scroll((scroll_offset, 0));
    f.render_widget(content_box, size);
}

fn draw_search_bar(query: &str, size: Rect, f: &mut Frame, focused: bool) {
    let input_hint = "Type your query here";

    let display_text: _ = if query.is_empty() {
        Paragraph::new(input_hint).style(Style::default().fg(Color::DarkGray))
    } else {
        Paragraph::new(String::from(query))
    };

    let search_box = display_text
        .block(Block::default().title("Search").borders(Borders::ALL))
        .style(if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        })
        .alignment(ratatui::layout::Alignment::Left);
    f.render_widget(search_box, size);
}

fn draw_file_preview(
    area: Rect,
    f: &mut Frame<'_>,
    preview_cache: &HashMap<PathBuf, Text<'static>>,
    selected_path: &Option<PathBuf>,
) {
    let text = selected_path
        .as_ref()
        .and_then(|path| preview_cache.get(path).cloned())
        .unwrap_or_else(|| Text::from("No Preview available"));

    let preview = Paragraph::new(text).block(
        Block::default()
            .title("Preview")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Gray)),
    );

    f.render_widget(preview, area);
}
