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

pub fn highlight_contents<'a>(file_path: &Path, content: &str) -> Text<'a> {
    let syntax = SS
        .find_syntax_for_file(file_path)
        .ok()
        .flatten()
        .unwrap_or_else(|| SS.find_syntax_plain_text());

    let mut h = HighlightLines::new(syntax, &TS.themes["base16-ocean.dark"]);

    let mut lines_to_render = Vec::new();
    let max_display_lines = 50;

    for line_str in content.lines().take(max_display_lines) {
        let ranges: Vec<(SyntectStyle, &str)> = h.highlight_line(line_str, &SS).unwrap_or_default();

        let mut spans: Vec<Span<'a>> = ranges
            .into_iter()
            .map(|(style, text)| {
                Span::styled(
                    text.to_string(),
                    Style::default().fg(convert_syntect_color(style.foreground)),
                )
            })
            .collect();
<<<<<<< HEAD
        spans.push(Span::styled(
            " ".repeat(prev_width as usize),
            Style::default(),
        ));
=======
>>>>>>> refs/remotes/origin/main

        lines_to_render.push(Line::from(spans));
    }

<<<<<<< HEAD
    while (lines_to_render.len() as u16) < prev_height {
        lines_to_render.push(Line::from(Span::styled(
            " ".repeat(prev_height as usize),
            Style::default(),
        )));
    }

=======
>>>>>>> refs/remotes/origin/main
    Text::from(lines_to_render)
}

fn convert_syntect_color(color: syntect::highlighting::Color) -> ratatui::style::Color {
    ratatui::style::Color::Rgb(color.r, color.g, color.b)
}
