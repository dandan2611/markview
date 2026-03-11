use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

pub struct SearchBar<'a> {
    pub query: &'a str,
}

impl Widget for SearchBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::default().bg(Color::Black).fg(Color::White);

        // Clear the line
        for x in area.left()..area.right() {
            buf[(x, area.y)].set_style(style).set_char(' ');
        }

        let line = Line::from(vec![
            Span::styled(
                "/",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(self.query.to_string(), style),
            Span::styled(
                "█",
                Style::default().fg(Color::White),
            ),
        ]);

        buf.set_line(area.left(), area.top(), &line, area.width);
    }
}
