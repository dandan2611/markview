use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone)]
pub struct LinkInfo {
    pub line_index: usize,
    #[allow(dead_code)]
    pub span_start: usize,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct CodeBlockRange {
    pub block_id: usize,
    pub start_line: usize,
    pub end_line: usize,
    #[allow(dead_code)]
    pub width: usize,
}

#[derive(Debug, Clone, Default)]
pub struct StyleStack {
    bold: u32,
    italic: u32,
    strikethrough: u32,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
}

impl StyleStack {
    pub fn push_bold(&mut self) {
        self.bold += 1;
    }

    pub fn pop_bold(&mut self) {
        self.bold = self.bold.saturating_sub(1);
    }

    pub fn push_italic(&mut self) {
        self.italic += 1;
    }

    pub fn pop_italic(&mut self) {
        self.italic = self.italic.saturating_sub(1);
    }

    pub fn push_strikethrough(&mut self) {
        self.strikethrough += 1;
    }

    pub fn pop_strikethrough(&mut self) {
        self.strikethrough = self.strikethrough.saturating_sub(1);
    }

    pub fn to_style(&self) -> Style {
        let mut style = Style::default();
        if let Some(fg) = self.fg {
            style = style.fg(fg);
        }
        if let Some(bg) = self.bg {
            style = style.bg(bg);
        }
        let mut mods = Modifier::empty();
        if self.bold > 0 {
            mods |= Modifier::BOLD;
        }
        if self.italic > 0 {
            mods |= Modifier::ITALIC;
        }
        if self.strikethrough > 0 {
            mods |= Modifier::CROSSED_OUT;
        }
        style.add_modifier(mods)
    }
}
