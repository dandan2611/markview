use std::path::PathBuf;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::text::Line;
use tokio::sync::mpsc;

use crate::event::AppEvent;
use crate::markdown::highlight::{HighlightRequest, HighlightResult, Highlighter};
use crate::markdown::parser;
use crate::markdown::types::{CodeBlockRange, LinkInfo};
use crate::search::SearchState;
use crate::theme::Theme;
use crate::ui::file_picker::FilePicker;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    FilePicker,
    Viewer,
    Search,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputSource {
    FileArg(PathBuf),
    Picker,
    Stdin,
}

pub struct App {
    pub mode: Mode,
    pub should_quit: bool,
    pub input_source: InputSource,
    pub theme: Theme,

    // Viewer state
    pub lines: Vec<Line<'static>>,
    pub links: Vec<LinkInfo>,
    pub code_blocks: Vec<CodeBlockRange>,
    pub scroll_offset: u16,
    pub viewport_height: u16,
    pub filename: String,
    pub word_count: usize,
    pub focused_link: Option<usize>,
    pub table_wrap: bool,

    // Search
    pub search: SearchState,

    // File picker
    pub file_picker: Option<FilePicker>,

    // Error popup
    pub error_message: Option<String>,

    // Highlight channel
    pub highlight_tx: Option<mpsc::UnboundedSender<AppEvent>>,

    // Current file path for watching
    pub current_file: Option<PathBuf>,
}

impl App {
    pub fn new(input_source: InputSource, theme: Theme) -> Self {
        let mode = match &input_source {
            InputSource::Picker => Mode::FilePicker,
            _ => Mode::Viewer,
        };

        let file_picker = if mode == Mode::FilePicker {
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            Some(FilePicker::new(&cwd))
        } else {
            None
        };

        Self {
            mode,
            should_quit: false,
            input_source,
            theme,
            lines: Vec::new(),
            links: Vec::new(),
            code_blocks: Vec::new(),
            scroll_offset: 0,
            viewport_height: 24,
            filename: String::new(),
            word_count: 0,
            focused_link: None,
            table_wrap: true,
            search: SearchState::default(),
            file_picker,
            error_message: None,
            highlight_tx: None,
            current_file: None,
        }
    }

    pub fn load_content(&mut self, content: &str, filename: &str, terminal_width: u16) {
        let result = parser::parse_markdown(content, &self.theme, terminal_width);
        self.lines = result.lines;
        self.links = result.links;
        self.code_blocks = result.code_blocks;
        self.word_count = result.word_count;
        self.filename = filename.to_string();
        self.focused_link = None;
        self.search.clear();

        // Queue syntax highlighting
        self.queue_highlighting(result.highlight_requests);
    }

    pub fn load_file(&mut self, path: &PathBuf, terminal_width: u16) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let filename = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "<unknown>".to_string());
        self.current_file = Some(path.clone());
        self.load_content(&content, &filename, terminal_width);
        Ok(())
    }

    fn queue_highlighting(&self, requests: Vec<HighlightRequest>) {
        if requests.is_empty() {
            return;
        }
        if let Some(ref tx) = self.highlight_tx {
            let tx = tx.clone();
            let theme_name = self.theme.syntect_theme_name().to_string();
            let code_bg = self.theme.palette.code_block_bg;
            let border_color = self.theme.palette.blockquote_border;

            tokio::spawn(async move {
                let highlighter = Highlighter::new(&theme_name, code_bg);
                for req in requests {
                    let highlighted =
                        highlighter.highlight(&req.code, &req.language, req.width, border_color);
                    let result = HighlightResult {
                        block_id: req.block_id,
                        lines: highlighted,
                    };
                    let _ = tx.send(AppEvent::HighlightDone(result));
                }
            });
        }
    }

    pub fn apply_highlight(&mut self, result: HighlightResult) {
        if let Some(block) = self.code_blocks.iter().find(|b| b.block_id == result.block_id) {
            let start = block.start_line;
            let end = block.end_line;
            let old_len = end - start;
            let new_len = result.lines.len();

            // Replace the placeholder lines with highlighted lines
            let mut new_lines: Vec<Line<'static>> = Vec::new();
            new_lines.extend_from_slice(&self.lines[..start]);
            new_lines.extend(result.lines);
            if end < self.lines.len() {
                new_lines.extend_from_slice(&self.lines[end..]);
            }
            self.lines = new_lines;

            // Adjust code block ranges for subsequent blocks
            if new_len != old_len {
                let diff = new_len as isize - old_len as isize;
                for block in &mut self.code_blocks {
                    if block.start_line > start {
                        block.start_line = (block.start_line as isize + diff) as usize;
                        block.end_line = (block.end_line as isize + diff) as usize;
                    }
                }
                // Adjust link line indices
                for link in &mut self.links {
                    if link.line_index >= end {
                        link.line_index = (link.line_index as isize + diff) as usize;
                    }
                }
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // Error popup dismissal
        if self.error_message.is_some() {
            self.error_message = None;
            if matches!(self.input_source, InputSource::FileArg(_)) {
                self.should_quit = true;
            } else if self.mode == Mode::Viewer {
                self.mode = Mode::FilePicker;
                if self.file_picker.is_none() {
                    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                    self.file_picker = Some(FilePicker::new(&cwd));
                }
            }
            return;
        }

        match self.mode {
            Mode::FilePicker => self.handle_picker_key(key),
            Mode::Viewer => self.handle_viewer_key(key),
            Mode::Search => self.handle_search_key(key),
        }
    }

    fn handle_picker_key(&mut self, key: KeyEvent) {
        let picker = match self.file_picker.as_mut() {
            Some(p) => p,
            None => return,
        };

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Char('j') | KeyCode::Down => picker.move_down(),
            KeyCode::Char('k') | KeyCode::Up => picker.move_up(),
            KeyCode::Char('g') | KeyCode::Home => picker.jump_top(),
            KeyCode::Char('G') | KeyCode::End => picker.jump_bottom(),
            KeyCode::Char('d') | KeyCode::PageDown => picker.page_down(10),
            KeyCode::Char('u') | KeyCode::PageUp => picker.page_up(10),
            KeyCode::Enter => {
                if let Some(path) = picker.enter_selected() {
                    // Open file
                    match self.load_file(&path, 80) {
                        Ok(()) => {
                            self.mode = Mode::Viewer;
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Failed to open file: {e}"));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_viewer_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                match self.input_source {
                    InputSource::FileArg(_) | InputSource::Stdin => {
                        self.should_quit = true;
                    }
                    InputSource::Picker => {
                        self.mode = Mode::FilePicker;
                        self.current_file = None;
                    }
                }
            }
            KeyCode::Char('j') | KeyCode::Down => self.scroll_down(1),
            KeyCode::Char('k') | KeyCode::Up => self.scroll_up(1),
            KeyCode::Char('d') | KeyCode::PageDown => {
                self.scroll_down(self.viewport_height / 2);
            }
            KeyCode::Char('u') | KeyCode::PageUp => {
                self.scroll_up(self.viewport_height / 2);
            }
            KeyCode::Char('g') | KeyCode::Home => {
                self.scroll_offset = 0;
            }
            KeyCode::Char('G') | KeyCode::End => {
                self.scroll_offset = self.max_scroll();
            }
            KeyCode::Char('/') => {
                self.mode = Mode::Search;
                self.search.active = true;
                self.search.query.clear();
            }
            KeyCode::Char('n') => {
                if !self.search.matches.is_empty() {
                    self.search.next_match();
                    self.scroll_to_search_match();
                }
            }
            KeyCode::Char('N') => {
                if !self.search.matches.is_empty() {
                    self.search.prev_match();
                    self.scroll_to_search_match();
                }
            }
            KeyCode::Tab => self.next_link(),
            KeyCode::BackTab => self.prev_link(),
            KeyCode::Enter => self.open_focused_link(),
            KeyCode::Char('w') => {
                self.table_wrap = !self.table_wrap;
            }
            _ => {}
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.search.clear();
                self.mode = Mode::Viewer;
            }
            KeyCode::Enter => {
                self.search.active = !self.search.query.is_empty();
                self.mode = Mode::Viewer;
                if self.search.active {
                    self.scroll_to_search_match();
                }
            }
            KeyCode::Backspace => {
                self.search.query.pop();
                self.search.search(&self.lines);
            }
            KeyCode::Char(c) => {
                self.search.query.push(c);
                self.search.search(&self.lines);
            }
            _ => {}
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                if self.mode == Mode::Viewer {
                    self.scroll_up(3);
                }
            }
            MouseEventKind::ScrollDown => {
                if self.mode == Mode::Viewer {
                    self.scroll_down(3);
                }
            }
            MouseEventKind::Down(_) => {
                if self.mode == Mode::Viewer {
                    // Try to detect link click based on mouse position
                    let click_line = self.scroll_offset as usize + mouse.row as usize;
                    for (i, link) in self.links.iter().enumerate() {
                        if link.line_index == click_line {
                            self.focused_link = Some(i);
                            self.open_focused_link();
                            break;
                        }
                    }
                } else if self.mode == Mode::FilePicker {
                    // Click to select in file picker
                    if let Some(ref mut picker) = self.file_picker {
                        let clicked_index = mouse.row as usize;
                        if clicked_index > 0 && clicked_index <= picker.entries.len() {
                            picker.list_state.select(Some(clicked_index - 1));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn scroll_down(&mut self, lines: u16) {
        let max = self.max_scroll();
        self.scroll_offset = (self.scroll_offset + lines).min(max);
    }

    fn scroll_up(&mut self, lines: u16) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    fn max_scroll(&self) -> u16 {
        crate::ui::viewer::max_scroll(self.lines.len(), self.viewport_height)
    }

    pub fn scroll_percent(&self) -> u16 {
        let max = self.max_scroll();
        if max == 0 {
            0
        } else {
            ((self.scroll_offset as u32 * 100) / max as u32) as u16
        }
    }

    fn next_link(&mut self) {
        if self.links.is_empty() {
            return;
        }
        self.focused_link = Some(match self.focused_link {
            Some(i) => (i + 1) % self.links.len(),
            None => 0,
        });
        self.scroll_to_link();
    }

    fn prev_link(&mut self) {
        if self.links.is_empty() {
            return;
        }
        self.focused_link = Some(match self.focused_link {
            Some(0) | None => self.links.len() - 1,
            Some(i) => i - 1,
        });
        self.scroll_to_link();
    }

    fn scroll_to_link(&mut self) {
        if let Some(idx) = self.focused_link {
            if let Some(link) = self.links.get(idx) {
                let line = link.line_index as u16;
                if line < self.scroll_offset {
                    self.scroll_offset = line;
                } else if line >= self.scroll_offset + self.viewport_height {
                    self.scroll_offset = line.saturating_sub(self.viewport_height / 2);
                }
            }
        }
    }

    fn scroll_to_search_match(&mut self) {
        if let Some(line) = self.search.current_line() {
            let line = line as u16;
            if line < self.scroll_offset || line >= self.scroll_offset + self.viewport_height {
                self.scroll_offset = line.saturating_sub(self.viewport_height / 2);
            }
        }
    }

    fn open_focused_link(&self) {
        if let Some(idx) = self.focused_link {
            if let Some(link) = self.links.get(idx) {
                let _ = open::that(&link.url);
            }
        }
    }

    pub fn link_line_indices(&self) -> Vec<usize> {
        self.links.iter().map(|l| l.line_index).collect()
    }
}
