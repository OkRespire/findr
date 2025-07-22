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
    pub static ref ss: SyntaxSet = SyntaxSet::load_defaults_newlines();
    pub static ref ts: ThemeSet = ThemeSet::load_defaults();
}

pub fn highlight_contents(file_path: &Path, content: &str) -> Text<'static> {
    let syntax = ss
        .find_syntax_for_file(file_path)
        .ok()
        .flatten()
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let mut h = HighlightLines::new(&syntax, &ts.themes["base16-ocean.dark"]);

    let mut lines = Vec::new();

    for line_str in content.lines().take(50) {
        let ranges: Vec<(SyntectStyle, &str)> = h.highlight_line(line_str, &ss).unwrap_or_default();

        let spans: Vec<Span<'static>> = ranges
            .into_iter()
            .map(|(style, text)| {
                Span::styled(
                    text.to_string(),
                    Style::default().fg(convert_syntect_color(style.foreground)),
                )
            })
            .collect();

        lines.push(Line::from(spans));
    }

    Text::from(lines)
}

fn convert_syntect_color(color: syntect::highlighting::Color) -> ratatui::style::Color {
    ratatui::style::Color::Rgb(color.r, color.g, color.b)
}
