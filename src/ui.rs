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
            draw_search_bar(&query, vertical_chunks[0], f);
            draw_content_box(&filtered_files, vertical_chunks[1], f)
        })?;

        // Handle key events
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char(c) => query.push(c),
                KeyCode::Backspace => {
                    query.pop();
                }
                KeyCode::Esc => break,
                _ => {}
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

fn draw_content_box(contents: &[PathBuf], size: Rect, f: &mut Frame<'_>) {
    let line_of_content: Vec<Line> = contents
        .iter()
        .map(|p| Line::from(Span::raw(p.display().to_string())))
        .collect();

    let content_box = Paragraph::new(Text::from(line_of_content))
        .block(Block::default().title("Files").borders(Borders::ALL));
    f.render_widget(content_box, size);
}

fn draw_search_bar(query: &str, size: Rect, f: &mut Frame) {
    let input_hint = "Type your query here";

    let display_text: _ = if query.is_empty() {
        Paragraph::new(input_hint).style(Style::default().fg(Color::DarkGray))
    } else {
        Paragraph::new(String::from(query))
    };

    let search_box = display_text
        .block(Block::default().title("Search").borders(Borders::ALL))
        .alignment(ratatui::layout::Alignment::Left);
    f.render_widget(search_box, size);
}
