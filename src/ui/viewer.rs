use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget, Wrap};

use crate::search::SearchState;

pub struct ViewerWidget<'a> {
    lines: &'a [Line<'static>],
    scroll_offset: u16,
    search: &'a SearchState,
    focused_link: Option<usize>,
    link_line_indices: &'a [usize],
}

impl<'a> ViewerWidget<'a> {
    pub fn new(
        lines: &'a [Line<'static>],
        scroll_offset: u16,
        search: &'a SearchState,
        focused_link: Option<usize>,
        link_line_indices: &'a [usize],
    ) -> Self {
        Self {
            lines,
            scroll_offset,
            search,
            focused_link,
            link_line_indices,
        }
    }
}

impl Widget for ViewerWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut display_lines: Vec<Line<'static>> = Vec::with_capacity(self.lines.len());

        for (line_idx, line) in self.lines.iter().enumerate() {
            let is_focused_link_line = self
                .focused_link
                .map(|li| self.link_line_indices.get(li) == Some(&line_idx))
                .unwrap_or(false);

            let search_ranges: Vec<(usize, usize, bool)> =
                if self.search.active && !self.search.query.is_empty() {
                    self.search
                        .matches
                        .iter()
                        .enumerate()
                        .filter(|(_, m)| m.line_index == line_idx)
                        .map(|(i, m)| (m.byte_start, m.byte_end, i == self.search.current_match))
                        .collect()
                } else {
                    Vec::new()
                };

            if search_ranges.is_empty() && !is_focused_link_line {
                display_lines.push(line.clone());
            } else if !search_ranges.is_empty() {
                let new_spans = apply_search_highlights(line, &search_ranges);
                display_lines.push(Line::from(new_spans));
            } else if is_focused_link_line {
                let new_spans = apply_link_focus(line);
                display_lines.push(Line::from(new_spans));
            } else {
                display_lines.push(line.clone());
            }
        }

        let paragraph = Paragraph::new(display_lines)
            .scroll((self.scroll_offset, 0))
            .wrap(Wrap { trim: false });

        paragraph.render(area, buf);
    }
}

fn apply_search_highlights(
    line: &Line<'static>,
    search_ranges: &[(usize, usize, bool)],
) -> Vec<Span<'static>> {
    let mut new_spans: Vec<Span<'static>> = Vec::new();
    let mut char_offset = 0;

    for span in &line.spans {
        let span_text = span.content.as_ref();
        let span_start = char_offset;
        let span_end = char_offset + span_text.len();

        let mut pos = 0;
        for &(match_start, match_end, is_current) in search_ranges {
            let overlap_start = match_start.max(span_start).saturating_sub(span_start);
            let overlap_end = match_end.min(span_end).saturating_sub(span_start);

            if overlap_start < overlap_end && overlap_start < span_text.len() {
                if pos < overlap_start {
                    new_spans.push(Span::styled(
                        span_text[pos..overlap_start].to_string(),
                        span.style,
                    ));
                }
                let hl_style = if is_current {
                    Style::default()
                        .bg(Color::Yellow)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().bg(Color::Yellow).fg(Color::Black)
                };
                let end = overlap_end.min(span_text.len());
                new_spans.push(Span::styled(
                    span_text[overlap_start..end].to_string(),
                    hl_style,
                ));
                pos = end;
            }
        }
        if pos < span_text.len() {
            new_spans.push(Span::styled(span_text[pos..].to_string(), span.style));
        }
        char_offset = span_end;
    }

    new_spans
}

fn apply_link_focus(line: &Line<'static>) -> Vec<Span<'static>> {
    let mut new_spans: Vec<Span<'static>> = Vec::new();
    for span in &line.spans {
        let has_underline = span.style.add_modifier.contains(Modifier::UNDERLINED);
        let is_link_color =
            span.style.fg == Some(Color::Blue) || span.style.fg == Some(Color::DarkGray);

        if has_underline || is_link_color {
            let style = Style::default()
                .fg(Color::Black)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD);
            new_spans.push(Span::styled(span.content.to_string(), style));
        } else {
            new_spans.push(span.clone());
        }
    }
    new_spans
}

pub fn max_scroll(total_lines: usize, viewport_height: u16) -> u16 {
    total_lines.saturating_sub(viewport_height as usize) as u16
}
