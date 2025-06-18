use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
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
use std::{error::Error, io, path::PathBuf};

enum Focus {
    SearchBar,
    Results,
}

pub fn run_app(
    contents: &Vec<PathBuf>,
    matcher: &mut nucleo::Matcher,
) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut query = String::new();
    let mut filtered_files = contents.clone();
    let mut buf = Vec::new();
    let mut focus = Focus::SearchBar;
    let mut selected_idx = 0;

    loop {
        buf.clear();
        terminal.draw(|f| {
            let size = f.area();
            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Search bar height
                    Constraint::Min(1),    // File list fills remaining space
                ])
                .split(size);
            draw_search_bar(
                &query,
                vertical_chunks[0],
                f,
                matches!(focus, Focus::SearchBar),
            );
            draw_content_box(
                &filtered_files,
                vertical_chunks[1],
                f,
                matches!(focus, Focus::Results),
                selected_idx,
            )
        })?;

        if let Event::Key(key) = event::read()? {
            match focus {
                Focus::SearchBar => match key.code {
                    KeyCode::Char(c) => query.push(c),
                    KeyCode::Backspace => {
                        query.pop();
                    }
                    KeyCode::Tab => focus = Focus::Results,
                    KeyCode::Esc => break,
                    _ => {}
                },
                Focus::Results => match key.code {
                    KeyCode::Up => {
                        if selected_idx > 0 {
                            selected_idx -= 1;
                        } else {
                            selected_idx = filtered_files.len() - 1
                        }
                    }
                    KeyCode::Down => {
                        if selected_idx + 1 < filtered_files.len() {
                            selected_idx += 1;
                        } else {
                            selected_idx = 0
                        }
                    }
                    KeyCode::BackTab => focus = Focus::SearchBar,
                    KeyCode::Esc => break,
                    _ => {}
                },
            }
        }

        let query_lower = &query.to_lowercase();
        let query_utf32 = Utf32Str::new(&query_lower, &mut buf);

        filtered_files = update_filtered_files(query_utf32, contents, matcher);
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

    let content_box = Paragraph::new(Text::from(line_of_content)).block(
        Block::default()
            .title("Files")
            .borders(Borders::ALL)
            .style(if focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            }),
    );
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
