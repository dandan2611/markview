use ratatui::text::Line;

#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub line_index: usize,
    pub byte_start: usize,
    pub byte_end: usize,
}

#[derive(Debug, Default)]
pub struct SearchState {
    pub query: String,
    pub matches: Vec<SearchMatch>,
    pub current_match: usize,
    pub active: bool,
}

impl SearchState {
    pub fn search(&mut self, lines: &[Line<'_>]) {
        self.matches.clear();
        self.current_match = 0;

        if self.query.is_empty() {
            return;
        }

        let query_lower = self.query.to_lowercase();

        for (line_idx, line) in lines.iter().enumerate() {
            let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
            let text_lower = text.to_lowercase();
            let mut start = 0;
            while let Some(pos) = text_lower[start..].find(&query_lower) {
                let abs_start = start + pos;
                self.matches.push(SearchMatch {
                    line_index: line_idx,
                    byte_start: abs_start,
                    byte_end: abs_start + self.query.len(),
                });
                start = abs_start + 1;
            }
        }
    }

    pub fn next_match(&mut self) {
        if !self.matches.is_empty() {
            self.current_match = (self.current_match + 1) % self.matches.len();
        }
    }

    pub fn prev_match(&mut self) {
        if !self.matches.is_empty() {
            self.current_match = if self.current_match == 0 {
                self.matches.len() - 1
            } else {
                self.current_match - 1
            };
        }
    }

    pub fn current_line(&self) -> Option<usize> {
        self.matches
            .get(self.current_match)
            .map(|m| m.line_index)
    }

    pub fn match_info(&self) -> Option<String> {
        if self.matches.is_empty() {
            None
        } else {
            Some(format!(
                "Match {}/{}",
                self.current_match + 1,
                self.matches.len()
            ))
        }
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.matches.clear();
        self.current_match = 0;
        self.active = false;
    }
}
