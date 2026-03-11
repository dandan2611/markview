use pulldown_cmark::{Alignment, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use crate::markdown::highlight::HighlightRequest;
use crate::markdown::types::{CodeBlockRange, LinkInfo, StyleStack};
use crate::theme::Theme;

pub struct ParseResult {
    pub lines: Vec<Line<'static>>,
    pub links: Vec<LinkInfo>,
    pub code_blocks: Vec<CodeBlockRange>,
    pub highlight_requests: Vec<HighlightRequest>,
    pub word_count: usize,
}

struct TableState {
    headers: Vec<Vec<Span<'static>>>,
    rows: Vec<Vec<Vec<Span<'static>>>>,
    alignments: Vec<Alignment>,
    current_row: Vec<Vec<Span<'static>>>,
    current_cell: Vec<Span<'static>>,
    in_header: bool,
}

pub fn parse_markdown(source: &str, theme: &Theme, terminal_width: u16) -> ParseResult {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_HEADING_ATTRIBUTES
        | Options::ENABLE_SMART_PUNCTUATION
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
        | Options::ENABLE_MATH;
    let parser = Parser::new_ext(source, options);

    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut links: Vec<LinkInfo> = Vec::new();
    let mut code_blocks: Vec<CodeBlockRange> = Vec::new();
    let mut highlight_requests: Vec<HighlightRequest> = Vec::new();
    let mut word_count: usize = 0;

    let mut style_stack = StyleStack::default();
    let mut current_spans: Vec<Span<'static>> = Vec::new();
    let mut heading_level: Option<u8> = None;
    let mut list_depth: usize = 0;
    let mut list_indices: Vec<Option<u64>> = Vec::new(); // None = unordered, Some(n) = ordered
    let mut blockquote_depth: usize = 0;
    let mut in_code_block = false;
    let mut code_block_lang = String::new();
    let mut code_block_content = String::new();
    let mut code_block_id: usize = 0;
    let mut link_url: Option<String> = None;
    let mut table_state: Option<TableState> = None;
    let mut footnote_labels: Vec<String> = Vec::new();
    let mut in_metadata_block = false;

    let events: Vec<Event> = parser.collect();

    for event in events {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    let lvl = match level {
                        HeadingLevel::H1 => 1,
                        HeadingLevel::H2 => 2,
                        HeadingLevel::H3 => 3,
                        HeadingLevel::H4 => 4,
                        HeadingLevel::H5 => 5,
                        HeadingLevel::H6 => 6,
                    };
                    heading_level = Some(lvl);
                    current_spans.clear();
                    style_stack = StyleStack::default();
                    style_stack.push_bold();
                    style_stack.fg = Some(theme.heading_fg(lvl));
                }
                Tag::Emphasis => {
                    style_stack.push_italic();
                }
                Tag::Strong => {
                    style_stack.push_bold();
                }
                Tag::Strikethrough => {
                    style_stack.push_strikethrough();
                }
                Tag::List(start) => {
                    list_depth += 1;
                    list_indices.push(start);
                }
                Tag::Item => {
                    current_spans.clear();
                    let indent = "  ".repeat(list_depth.saturating_sub(1));
                    let bq_prefix = build_blockquote_prefix(blockquote_depth, theme);

                    let bullet = if let Some(idx) = list_indices.last_mut() {
                        if let Some(ref mut n) = idx {
                            let b = format!("{n}. ");
                            *n += 1;
                            b
                        } else {
                            "• ".to_string()
                        }
                    } else {
                        "• ".to_string()
                    };

                    for span in bq_prefix {
                        current_spans.push(span);
                    }
                    current_spans.push(Span::raw(format!("{indent}{bullet}")));
                }
                Tag::BlockQuote(_) => {
                    blockquote_depth += 1;
                }
                Tag::CodeBlock(kind) => {
                    in_code_block = true;
                    code_block_content.clear();
                    code_block_lang = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                        pulldown_cmark::CodeBlockKind::Indented => String::new(),
                    };
                }
                Tag::Link { dest_url, .. } => {
                    link_url = Some(dest_url.to_string());
                    style_stack.fg = Some(theme.palette.link_fg);
                }
                Tag::Image { dest_url, .. } => {
                    let url = dest_url.to_string();
                    link_url = Some(url);
                }
                Tag::Paragraph => {
                    current_spans.clear();
                    if blockquote_depth > 0 {
                        let bq_prefix = build_blockquote_prefix(blockquote_depth, theme);
                        for span in bq_prefix {
                            current_spans.push(span);
                        }
                    }
                }
                Tag::Table(alignments) => {
                    table_state = Some(TableState {
                        headers: Vec::new(),
                        rows: Vec::new(),
                        alignments: alignments.to_vec(),
                        current_row: Vec::new(),
                        current_cell: Vec::new(),
                        in_header: false,
                    });
                }
                Tag::TableHead => {
                    if let Some(ref mut ts) = table_state {
                        ts.in_header = true;
                        ts.current_row.clear();
                    }
                }
                Tag::TableRow => {
                    if let Some(ref mut ts) = table_state {
                        ts.current_row.clear();
                    }
                }
                Tag::TableCell => {
                    if let Some(ref mut ts) = table_state {
                        ts.current_cell.clear();
                    }
                }
                Tag::FootnoteDefinition(label) => {
                    let label_str = label.to_string();
                    let index = footnote_labels
                        .iter()
                        .position(|l| l == &label_str)
                        .map(|i| i + 1)
                        .unwrap_or_else(|| {
                            footnote_labels.push(label_str);
                            footnote_labels.len()
                        });
                    current_spans.clear();
                    current_spans.push(Span::styled(
                        format!("[{index}]: "),
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    ));
                }
                Tag::MetadataBlock(_) => {
                    in_metadata_block = true;
                }
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Heading(_) => {
                    if let Some(lvl) = heading_level.take() {
                        let fg = theme.heading_fg(lvl);
                        let heading_style = Style::default()
                            .fg(fg)
                            .add_modifier(Modifier::BOLD);

                        // Build prefix: "# ", "## ", etc.
                        let prefix = format!("{} ", "#".repeat(lvl as usize));
                        let prefix_len = prefix.len();
                        let mut spans = vec![Span::styled(prefix, heading_style)];

                        // Append content spans
                        let mut content_len: usize = 0;
                        for span in current_spans.drain(..) {
                            content_len += span.content.len();
                            spans.push(Span::styled(
                                span.content.to_string(),
                                span.style
                                    .fg(fg)
                                    .add_modifier(Modifier::BOLD),
                            ));
                        }
                        lines.push(Line::from(spans));

                        // Add underline if applicable
                        if let Some(ch) = Theme::heading_underline_char(lvl) {
                            let total_width = prefix_len + content_len;
                            let underline_color = theme.heading_underline_color();
                            let repeat_count = if ch.len() > 1 {
                                // "· " is 3 bytes (2-byte char + space), visually 2 columns
                                total_width / 2
                            } else {
                                total_width
                            };
                            let underline = ch.repeat(repeat_count);
                            lines.push(Line::from(Span::styled(
                                underline,
                                Style::default().fg(underline_color),
                            )));
                        }

                        lines.push(Line::from(""));
                    }
                    style_stack = StyleStack::default();
                }
                TagEnd::Emphasis => {
                    style_stack.pop_italic();
                }
                TagEnd::Strong => {
                    style_stack.pop_bold();
                }
                TagEnd::Strikethrough => {
                    style_stack.pop_strikethrough();
                }
                TagEnd::List(_) => {
                    list_depth = list_depth.saturating_sub(1);
                    list_indices.pop();
                    if list_depth == 0 {
                        lines.push(Line::from(""));
                    }
                }
                TagEnd::Item => {
                    if !current_spans.is_empty() {
                        lines.push(Line::from(current_spans.drain(..).collect::<Vec<_>>()));
                    }
                }
                TagEnd::BlockQuote(_) => {
                    blockquote_depth = blockquote_depth.saturating_sub(1);
                    if blockquote_depth == 0 {
                        lines.push(Line::from(""));
                    }
                }
                TagEnd::CodeBlock => {
                    in_code_block = false;
                    let code = std::mem::take(&mut code_block_content);
                    let bg = theme.palette.code_block_bg;
                    let border_color = theme.palette.blockquote_border;
                    let border_style = Style::default().fg(border_color).bg(bg);

                    // Compute block_width: longest line + 2 (padding), capped to terminal_width - 2 (borders)
                    let max_line_len = if code.is_empty() {
                        0
                    } else {
                        code.lines().map(|l| l.len()).max().unwrap_or(0)
                    };
                    let block_width = (max_line_len + 2).min((terminal_width as usize).saturating_sub(2));

                    let start_line = lines.len();

                    // Top border
                    let top = format!("┌{}┐", "─".repeat(block_width));
                    lines.push(Line::from(Span::styled(top, border_style)));

                    // Code lines with side borders
                    let inner_width = block_width.saturating_sub(2);
                    let code_lines_vec: Vec<&str> = if code.is_empty() {
                        vec![""]
                    } else {
                        code.lines().collect()
                    };
                    for code_line in &code_lines_vec {
                        let content_len = code_line.len();
                        let padding = if content_len < inner_width {
                            " ".repeat(inner_width - content_len)
                        } else {
                            String::new()
                        };
                        let spans = vec![
                            Span::styled("│ ", border_style),
                            Span::styled(
                                code_line.to_string(),
                                Style::default().bg(bg),
                            ),
                            Span::styled(padding, Style::default().bg(bg)),
                            Span::styled(" │", border_style),
                        ];
                        lines.push(Line::from(spans));
                    }

                    // Bottom border
                    let bottom = format!("└{}┘", "─".repeat(block_width));
                    lines.push(Line::from(Span::styled(bottom, border_style)));

                    let end_line = lines.len();

                    let block_id = code_block_id;
                    code_block_id += 1;

                    code_blocks.push(CodeBlockRange {
                        block_id,
                        start_line,
                        end_line,
                        width: block_width,
                    });

                    let lang = std::mem::take(&mut code_block_lang);
                    highlight_requests.push(HighlightRequest {
                        block_id,
                        code,
                        language: lang,
                        width: block_width,
                    });

                    lines.push(Line::from(""));
                }
                TagEnd::Link => {
                    if let Some(url) = link_url.take() {
                        // Store link info
                        let line_index = lines.len(); // will be on current line being built
                        let span_start = current_spans.len().saturating_sub(1);
                        links.push(LinkInfo {
                            line_index,
                            span_start,
                            url: url.clone(),
                        });

                        // Append URL in dim
                        current_spans.push(Span::styled(
                            format!(" ({url})"),
                            Style::default().fg(Color::DarkGray),
                        ));
                    }
                    style_stack.fg = None;
                }
                TagEnd::Image => {
                    // Image already handled in text event
                    link_url = None;
                }
                TagEnd::Paragraph => {
                    if !current_spans.is_empty() {
                        lines.push(Line::from(current_spans.drain(..).collect::<Vec<_>>()));
                    }
                    lines.push(Line::from(""));
                }
                TagEnd::Table => {
                    if let Some(ts) = table_state.take() {
                        render_table(&ts, &mut lines, terminal_width);
                        lines.push(Line::from(""));
                    }
                }
                TagEnd::TableHead => {
                    if let Some(ref mut ts) = table_state {
                        ts.headers = ts.current_row.drain(..).collect();
                        ts.in_header = false;
                    }
                }
                TagEnd::TableRow => {
                    if let Some(ref mut ts) = table_state {
                        let row: Vec<Vec<Span<'static>>> = ts.current_row.drain(..).collect();
                        ts.rows.push(row);
                    }
                }
                TagEnd::TableCell => {
                    if let Some(ref mut ts) = table_state {
                        let cell = std::mem::take(&mut ts.current_cell);
                        ts.current_row.push(cell);
                    }
                }
                TagEnd::FootnoteDefinition => {
                    if !current_spans.is_empty() {
                        lines.push(Line::from(current_spans.drain(..).collect::<Vec<_>>()));
                    }
                }
                TagEnd::MetadataBlock(_) => {
                    // Silently discard metadata blocks (YAML front matter)
                    in_metadata_block = false;
                }
                _ => {}
            },
            Event::Text(text) => {
                if in_metadata_block {
                    continue;
                }
                let text_str = text.to_string();
                word_count += text_str.split_whitespace().count();

                if in_code_block {
                    code_block_content.push_str(&text_str);
                } else if let Some(ref mut ts) = table_state {
                    ts.current_cell.push(Span::styled(
                        text_str,
                        style_stack.to_style(),
                    ));
                } else if link_url.is_some() {
                    // Check if we're in an image tag
                    let style = Style::default()
                        .fg(theme.palette.link_fg)
                        .add_modifier(Modifier::UNDERLINED);
                    current_spans.push(Span::styled(text_str, style));
                } else {
                    current_spans.push(Span::styled(text_str, style_stack.to_style()));
                }
            }
            Event::Code(code) => {
                let text_str = code.to_string();
                word_count += text_str.split_whitespace().count();

                if let Some(ref mut ts) = table_state {
                    ts.current_cell.push(Span::styled(
                        text_str,
                        Style::default().bg(theme.palette.inline_code_bg),
                    ));
                } else {
                    current_spans.push(Span::styled(
                        text_str,
                        Style::default().bg(theme.palette.inline_code_bg),
                    ));
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if !current_spans.is_empty() {
                    lines.push(Line::from(current_spans.drain(..).collect::<Vec<_>>()));
                }
                if blockquote_depth > 0 {
                    let bq_prefix = build_blockquote_prefix(blockquote_depth, theme);
                    for span in bq_prefix {
                        current_spans.push(span);
                    }
                }
            }
            Event::Rule => {
                let rule = "─".repeat(terminal_width as usize);
                lines.push(Line::from(Span::styled(
                    rule,
                    Style::default().fg(Color::DarkGray),
                )));
                lines.push(Line::from(""));
            }
            Event::TaskListMarker(checked) => {
                let marker = if checked { "☑ " } else { "☐ " };
                current_spans.push(Span::raw(marker.to_string()));
            }
            Event::FootnoteReference(label) => {
                let label_str = label.to_string();
                let index = footnote_labels
                    .iter()
                    .position(|l| l == &label_str)
                    .map(|i| i + 1)
                    .unwrap_or_else(|| {
                        footnote_labels.push(label_str);
                        footnote_labels.len()
                    });
                current_spans.push(Span::styled(
                    format!("[{index}]"),
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                ));
            }
            Event::InlineMath(math) => {
                let text_str = math.to_string();
                current_spans.push(Span::styled(
                    text_str,
                    Style::default().bg(theme.palette.inline_code_bg),
                ));
            }
            Event::DisplayMath(math) => {
                let text_str = math.to_string();
                let bg = theme.palette.code_block_bg;
                for line_text in text_str.lines() {
                    lines.push(Line::from(Span::styled(
                        format!("  {line_text}"),
                        Style::default().bg(bg),
                    )));
                }
                lines.push(Line::from(""));
            }
            _ => {}
        }
    }

    // Flush any remaining spans
    if !current_spans.is_empty() {
        lines.push(Line::from(current_spans));
    }

    ParseResult {
        lines,
        links,
        code_blocks,
        highlight_requests,
        word_count,
    }
}

fn build_blockquote_prefix(depth: usize, theme: &Theme) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    for _ in 0..depth {
        spans.push(Span::styled(
            "│ ",
            Style::default().fg(theme.palette.blockquote_border),
        ));
    }
    spans
}

fn render_table(ts: &TableState, lines: &mut Vec<Line<'static>>, terminal_width: u16) {
    let num_cols = ts.headers.len();
    if num_cols == 0 {
        return;
    }

    // Compute column widths (minimum from content)
    let mut col_widths: Vec<usize> = vec![0; num_cols];

    for (i, header) in ts.headers.iter().enumerate() {
        let w: usize = header.iter().map(|s| s.content.len()).sum();
        col_widths[i] = col_widths[i].max(w);
    }
    for row in &ts.rows {
        for (i, cell) in row.iter().enumerate() {
            if i < num_cols {
                let w: usize = cell.iter().map(|s| s.content.len()).sum();
                col_widths[i] = col_widths[i].max(w);
            }
        }
    }

    // Cap columns to fit terminal
    let total: usize = col_widths.iter().sum::<usize>() + num_cols + 1; // borders
    if total > terminal_width as usize && terminal_width > 0 {
        let available = (terminal_width as usize).saturating_sub(num_cols + 1);
        let current_total: usize = col_widths.iter().sum();
        if current_total > 0 {
            for w in &mut col_widths {
                *w = (*w * available) / current_total;
                if *w == 0 {
                    *w = 1;
                }
            }
        }
    }

    // Top border: ┌─┬─┐
    let top = build_table_border(&col_widths, '┌', '┬', '┐');
    lines.push(Line::from(Span::raw(top)));

    // Header row
    let header_line = build_table_row(&ts.headers, &col_widths, &ts.alignments, true);
    lines.push(header_line);

    // Separator: ├─┼─┤
    let sep = build_table_border(&col_widths, '├', '┼', '┤');
    lines.push(Line::from(Span::raw(sep)));

    // Data rows
    for row in &ts.rows {
        let row_line = build_table_row(row, &col_widths, &ts.alignments, false);
        lines.push(row_line);
    }

    // Bottom border: └─┴─┘
    let bottom = build_table_border(&col_widths, '└', '┴', '┘');
    lines.push(Line::from(Span::raw(bottom)));
}

fn build_table_border(col_widths: &[usize], left: char, mid: char, right: char) -> String {
    let mut s = String::new();
    s.push(left);
    for (i, w) in col_widths.iter().enumerate() {
        for _ in 0..(*w + 2) {
            s.push('─');
        }
        if i < col_widths.len() - 1 {
            s.push(mid);
        }
    }
    s.push(right);
    s
}

fn build_table_row(
    cells: &[Vec<Span<'static>>],
    col_widths: &[usize],
    alignments: &[Alignment],
    is_header: bool,
) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    spans.push(Span::raw("│"));

    for (i, width) in col_widths.iter().enumerate() {
        let cell_text: String = cells
            .get(i)
            .map(|c| c.iter().map(|s| s.content.as_ref()).collect::<String>())
            .unwrap_or_default();

        let alignment = alignments.get(i).copied().unwrap_or(Alignment::None);
        let padded = pad_cell(&cell_text, *width, alignment);

        let style = if is_header {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        spans.push(Span::styled(format!(" {padded} "), style));
        spans.push(Span::raw("│"));
    }

    Line::from(spans)
}

fn pad_cell(text: &str, width: usize, alignment: Alignment) -> String {
    let len = text.len();
    if len >= width {
        return text[..width].to_string();
    }
    let padding = width - len;
    match alignment {
        Alignment::Right => format!("{}{}", " ".repeat(padding), text),
        Alignment::Center => {
            let left = padding / 2;
            let right = padding - left;
            format!("{}{}{}", " ".repeat(left), text, " ".repeat(right))
        }
        _ => format!("{}{}", text, " ".repeat(padding)),
    }
}
