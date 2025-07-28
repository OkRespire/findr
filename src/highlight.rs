use syntect::{
    easy::HighlightLines,
    highlighting::{Style as SyntectStyle, ThemeSet},
    parsing::SyntaxSet,
};

use ratatui::{
    style::Style,
    text::{Line, Span, Text},
};

use std::path::Path;

lazy_static::lazy_static! {
    pub static ref SS: SyntaxSet = SyntaxSet::load_defaults_newlines();
    pub static ref TS: ThemeSet = ThemeSet::load_defaults();
}

pub fn highlight_contents(
    file_path: &Path,
    content: &str,
    prev_height: u16,
    prev_width: u16,
) -> Text<'static> {
    let syntax = SS
        .find_syntax_for_file(file_path)
        .ok()
        .flatten()
        .unwrap_or_else(|| SS.find_syntax_plain_text());

    let mut h = HighlightLines::new(syntax, &TS.themes["base16-ocean.dark"]);
    let default_theme_bg_color =
        convert_syntect_color(TS.themes["base16-ocean.dark"].settings.background.unwrap());

    let mut lines_to_render = Vec::new();
    let max_display_lines = 50;

    for line_str in content.lines().take(max_display_lines) {
        let ranges: Vec<(SyntectStyle, &str)> = h.highlight_line(line_str, &SS).unwrap_or_default();

        let mut spans: Vec<Span<'static>> = ranges
            .into_iter()
            .map(|(style, text)| {
                Span::styled(
                    text.to_string(),
                    Style::default().fg(convert_syntect_color(style.foreground)),
                )
            })
            .collect();
        let curr_line_width = spans.iter().map(|s| s.width()).sum::<usize>() as u16;
        if curr_line_width < prev_width {
            let pad_len = prev_width - curr_line_width;
            // let padding_bg_col = spans
            //     .last()
            //     .and_then(|last_span| last_span.style.bg)
            //     .unwrap_or(default_theme_bg_color);
            spans.push(Span::styled(" ".repeat(pad_len as usize), Style::default()));
        }

        lines_to_render.push(Line::from(spans));
    }

    while (lines_to_render.len() as u16) < prev_height {
        lines_to_render.push(Line::from(Span::styled(
            " ".repeat(prev_width as usize),
            Style::default(),
        )));
    }

    Text::from(lines_to_render)
}

fn convert_syntect_color(color: syntect::highlighting::Color) -> ratatui::style::Color {
    ratatui::style::Color::Rgb(color.r, color.g, color.b)
}
