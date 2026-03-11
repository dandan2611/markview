use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;

#[derive(Debug, Clone)]
pub struct HighlightRequest {
    pub block_id: usize,
    pub code: String,
    pub language: String,
    pub width: usize,
}

#[derive(Debug, Clone)]
pub struct HighlightResult {
    pub block_id: usize,
    pub lines: Vec<Line<'static>>,
}

pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme: Theme,
    code_bg: Color,
}

impl Highlighter {
    pub fn new(theme_name: &str, code_bg: Color) -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        let theme = theme_set
            .themes
            .get(theme_name)
            .cloned()
            .unwrap_or_else(|| theme_set.themes["base16-ocean.dark"].clone());
        Self {
            syntax_set,
            theme,
            code_bg,
        }
    }

    pub fn highlight(
        &self,
        code: &str,
        language: &str,
        block_width: usize,
        border_color: Color,
    ) -> Vec<Line<'static>> {
        use syntect::easy::HighlightLines;

        let syntax = self
            .syntax_set
            .find_syntax_by_token(language)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let mut h = HighlightLines::new(syntax, &self.theme);
        let border_style = Style::default().fg(border_color).bg(self.code_bg);

        let mut result = Vec::new();

        // Top border
        let top = format!("┌{}┐", "─".repeat(block_width));
        result.push(Line::from(Span::styled(top, border_style)));

        let code_lines: Vec<&str> = if code.is_empty() {
            vec![""]
        } else {
            code.lines().collect()
        };

        for line_str in &code_lines {
            let regions = h
                .highlight_line(line_str, &self.syntax_set)
                .unwrap_or_default();

            let mut spans: Vec<Span<'static>> = Vec::new();
            spans.push(Span::styled("│ ", border_style));

            let mut content_width: usize = 0;
            for (style, text) in regions {
                let fg = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                content_width += text.len();
                spans.push(Span::styled(
                    text.to_string(),
                    Style::default().fg(fg).bg(self.code_bg),
                ));
            }

            // Pad to block_width - 2 (for the ` ` padding on each side)
            let inner_width = block_width.saturating_sub(2);
            if content_width < inner_width {
                spans.push(Span::styled(
                    " ".repeat(inner_width - content_width),
                    Style::default().bg(self.code_bg),
                ));
            }

            spans.push(Span::styled(" │", border_style));
            result.push(Line::from(spans));
        }

        // Bottom border
        let bottom = format!("└{}┘", "─".repeat(block_width));
        result.push(Line::from(Span::styled(bottom, border_style)));

        result
    }
}
