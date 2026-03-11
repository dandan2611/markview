use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum FileEntry {
    ParentDir,
    Directory(String),
    File(String),
}

impl FileEntry {
    pub fn display_name(&self) -> String {
        match self {
            FileEntry::ParentDir => "📁 ..".to_string(),
            FileEntry::Directory(name) => format!("📁 {name}/"),
            FileEntry::File(name) => format!("📄 {name}"),
        }
    }
}

pub struct FilePicker {
    pub current_dir: PathBuf,
    pub entries: Vec<FileEntry>,
    pub list_state: ListState,
}

impl FilePicker {
    pub fn new(dir: &Path) -> Self {
        let mut picker = Self {
            current_dir: dir.to_path_buf(),
            entries: Vec::new(),
            list_state: ListState::default(),
        };
        picker.scan_directory();
        picker
    }

    pub fn scan_directory(&mut self) {
        self.entries.clear();

        // Add parent dir entry (unless at root)
        if self.current_dir.parent().is_some() {
            self.entries.push(FileEntry::ParentDir);
        }

        let mut dirs: Vec<String> = Vec::new();
        let mut files: Vec<String> = Vec::new();

        if let Ok(read_dir) = std::fs::read_dir(&self.current_dir) {
            for entry in read_dir.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') {
                    continue; // Skip hidden files
                }
                if let Ok(ft) = entry.file_type() {
                    if ft.is_dir() {
                        dirs.push(name);
                    } else if name.ends_with(".md") || name.ends_with(".markdown") {
                        files.push(name);
                    }
                }
            }
        }

        dirs.sort();
        files.sort();

        for d in dirs {
            self.entries.push(FileEntry::Directory(d));
        }
        for f in files {
            self.entries.push(FileEntry::File(f));
        }

        if !self.entries.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
    }

    pub fn selected_entry(&self) -> Option<&FileEntry> {
        self.list_state.selected().and_then(|i| self.entries.get(i))
    }

    pub fn move_up(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            if selected > 0 {
                self.list_state.select(Some(selected - 1));
            }
        }
    }

    pub fn move_down(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            if selected + 1 < self.entries.len() {
                self.list_state.select(Some(selected + 1));
            }
        }
    }

    pub fn jump_top(&mut self) {
        if !self.entries.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    pub fn jump_bottom(&mut self) {
        if !self.entries.is_empty() {
            self.list_state.select(Some(self.entries.len() - 1));
        }
    }

    pub fn page_down(&mut self, page_size: usize) {
        if let Some(selected) = self.list_state.selected() {
            let new = (selected + page_size).min(self.entries.len().saturating_sub(1));
            self.list_state.select(Some(new));
        }
    }

    pub fn page_up(&mut self, page_size: usize) {
        if let Some(selected) = self.list_state.selected() {
            let new = selected.saturating_sub(page_size);
            self.list_state.select(Some(new));
        }
    }

    pub fn enter_selected(&mut self) -> Option<PathBuf> {
        let entry = self.selected_entry()?.clone();
        match entry {
            FileEntry::ParentDir => {
                if let Some(parent) = self.current_dir.parent() {
                    self.current_dir = parent.to_path_buf();
                    self.scan_directory();
                }
                None
            }
            FileEntry::Directory(name) => {
                self.current_dir = self.current_dir.join(&name);
                self.scan_directory();
                None
            }
            FileEntry::File(name) => Some(self.current_dir.join(&name)),
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let title = format!(
            " ▶ Select a Markdown file ({}) ",
            self.current_dir.display()
        );

        let items: Vec<ListItem> = self
            .entries
            .iter()
            .map(|e| ListItem::new(Line::from(Span::raw(e.display_name()))))
            .collect();

        let list = List::new(items)
            .block(Block::default().title(title).borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        StatefulWidget::render(list, area, buf, &mut self.list_state);
    }
}
