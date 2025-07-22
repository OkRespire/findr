use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use edit;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Color,
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear as RatatuiClear, Paragraph},
};

use nucleo::{Matcher, Utf32Str};
use std::{collections::HashMap, error::Error, io, path::PathBuf};

use crate::highlight::highlight_contents;

struct AppState {
    query: String,
    filtered_files: Vec<(PathBuf, String, Vec<u32>)>,
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
            draw_search_bar(
                &state,
                vertical_chunks[0],
                f,
                matches!(state.focus, Focus::SearchBar),
            );
            draw_content_box(
                &state,
                horizontal_chunks[0],
                f,
                matches!(state.focus, Focus::Results),
            );

            draw_file_preview(horizontal_chunks[1], f, &state);
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
        let query_utf32 = Utf32Str::new(&query_lower, &mut buf);

        state.filtered_files = update_filtered_files(query_utf32, all_files, matcher);

        // Update filtered files if query changed
        if state.query != prev_query {
            let query_lower = &state.query.to_lowercase();
            let query_utf32 = Utf32Str::new(&query_lower, &mut buf);
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
                let highlighted = highlight_contents(path, &content);
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

/// Uses nucleo to score files and ensures it is going in descending order, through the score
fn update_filtered_files(
    query_utf32: Utf32Str,
    contents: &[PathBuf],
    matcher: &mut Matcher,
) -> Vec<(PathBuf, String, Vec<u32>)> {
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
    scored_files
        .into_iter()
        .map(|(_, path, n, i)| (path, n, i))
        .collect::<Vec<(PathBuf, String, Vec<u32>)>>()
}

fn draw_content_box(app_state: &AppState, size: Rect, f: &mut Frame<'_>, focused: bool) {
    let line_of_content: Vec<Line> = app_state
        .filtered_files
        .iter()
        .enumerate()
        .map(|(i, (_, n, v))| {
            // p = path
            // n = name
            // v = index vector

            let highlights: std::collections::HashSet<u32> = v.iter().cloned().collect();

            let mut spans = Vec::new();

            for (idx, ch) in n.chars().enumerate() {
                let style = if highlights.contains(&(idx as u32)) {
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(ratatui::style::Modifier::BOLD)
                } else {
                    Style::default()
                };
                spans.push(Span::styled(ch.to_string(), style));
            }

            if i == app_state.selected_idx {
                let selected_style = Style::default().bg(Color::White).fg(Color::Black);
                let line = Line::from(spans);
                let selected_spans = line
                    .spans
                    .iter()
                    .map(|span| Span::styled(span.content.clone(), selected_style))
                    .collect::<Vec<_>>();
                Line::from(selected_spans)
            } else {
                Line::from(spans)
            }
        })
        .collect();
    let title_text = format!("Results ({})", app_state.filtered_files.len());

    let content_box = Paragraph::new(Text::from(line_of_content))
        .block(
            Block::default()
                .title(title_text)
                .borders(Borders::ALL)
                .style(if focused {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                }),
        )
        .scroll((app_state.scroll_offset, 0));
    f.render_widget(content_box, size);
}

fn draw_search_bar(app_state: &AppState, size: Rect, f: &mut Frame, focused: bool) {
    let input_hint = "Type your query here";

    let display_text: _ = if app_state.query.is_empty() {
        Paragraph::new(input_hint).style(Style::default().fg(Color::DarkGray))
    } else {
        Paragraph::new(String::from(&app_state.query))
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

fn draw_file_preview(area: Rect, f: &mut Frame<'_>, app_state: &AppState) {
    let text = app_state
        .selected_path
        .as_ref()
        .and_then(|path| app_state.preview_cache.get(path).cloned())
        .unwrap_or_else(|| Text::from(""));
    let path_title = if let Some(path_name) = &app_state.selected_path {
        path_name.to_string_lossy().into_owned()
    } else {
        "No directory selected".to_string()
    };

    let preview = Paragraph::new(text).block(
        Block::default()
            .title(path_title)
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Gray).bg(Color::Black)),
    );

    f.render_widget(RatatuiClear, area);
    f.render_widget(preview, area);
}
