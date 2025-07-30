use crate::ui::appstate::{AppState, Focus};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Color,
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear as RatatuiClear, Paragraph},
};

pub fn draw_ui(f: &mut Frame<'_>, state: &mut AppState) {
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

    f.render_widget(RatatuiClear, search_chunk);
    f.render_widget(RatatuiClear, content_chunk);
    f.render_widget(RatatuiClear, preview_chunk);

    let preview_block_for_calc = Block::default().borders(Borders::ALL);
    let inner_preview_area = preview_block_for_calc.inner(preview_chunk);

    state.curr_preview_width = inner_preview_area.width;
    state.curr_preview_height = inner_preview_area.height;
    draw_search_bar(
        state,
        search_chunk,
        f,
        matches!(state.focus, Focus::SearchBar),
    );
    draw_content_box(
        state,
        content_chunk,
        f,
        matches!(state.focus, Focus::Results),
    );

    draw_file_preview(preview_chunk, f, state);
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

    let display_text = if app_state.query.is_empty() {
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
        .unwrap_or_else(|| {
            let mut lines = Vec::new();
            lines.push(ratatui::text::Line::from(ratatui::text::Span::styled(
                "No Preview available",
                Style::default().fg(Color::DarkGray),
            )));
            while (lines.len() as u16) < app_state.curr_preview_height {
                lines.push(ratatui::text::Line::from(ratatui::text::Span::styled(
                    " ".repeat(app_state.curr_preview_width as usize),
                    Style::default(),
                )));
            }
            Text::from(lines)
        });

    let path_title = if let Some(path_name) = &app_state.selected_path {
        path_name.to_string_lossy().into_owned()
    } else {
        "No directory selected".to_string()
    };

    let preview = Paragraph::new(text).block(
        Block::default()
            .title(path_title)
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Gray).bg(Color::Reset)),
    );

    f.render_widget(preview, area);
}
