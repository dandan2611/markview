use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use crate::app::Mode;

pub struct StatusBar<'a> {
    pub filename: &'a str,
    pub line_count: usize,
    pub word_count: usize,
    pub scroll_percent: u16,
    pub mode: &'a Mode,
    pub table_wrap: bool,
    pub search_info: Option<String>,
}

impl Widget for StatusBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::default()
            .bg(Color::DarkGray)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD);

        // Clear the area
        for x in area.left()..area.right() {
            buf[(x, area.y)].set_style(style).set_char(' ');
        }

        let mode_str = match self.mode {
            Mode::FilePicker => "PICKER",
            Mode::Viewer => "VIEWER",
            Mode::Search => "SEARCH",
        };

        let hints = match self.mode {
            Mode::FilePicker => "q:quit ↑↓:nav ↵:open",
            Mode::Viewer => "q:back /:search w:wrap",
            Mode::Search => "↵:confirm esc:cancel",
        };

        let mut parts: Vec<String> = Vec::new();
        parts.push(format!(" {}", self.filename));
        parts.push(format!("{} lines", self.line_count));
        parts.push(format!("{} words", self.word_count));
        parts.push(format!("{}%", self.scroll_percent));
        parts.push(mode_str.to_string());

        if matches!(self.mode, Mode::Viewer) {
            if self.table_wrap {
                parts.push("[wrap]".to_string());
            } else {
                parts.push("[hscroll]".to_string());
            }
        }

        if let Some(ref search_info) = self.search_info {
            parts.push(search_info.clone());
        }

        parts.push(hints.to_string());

        let text = parts.join(" │ ");

        let line = Line::from(Span::styled(text, style));
        let x = area.left();
        let y = area.top();
        buf.set_line(x, y, &line, area.width);
    }
}
