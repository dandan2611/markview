use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

#[derive(Debug, Clone)]
pub struct ColorPalette {
    pub link_fg: Color,
    pub inline_code_bg: Color,
    pub code_block_bg: Color,
    pub blockquote_border: Color,
    #[allow(dead_code)]
    pub search_highlight: Color,
    #[allow(dead_code)]
    pub search_current: Color,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub mode: ThemeMode,
    pub palette: ColorPalette,
}

impl Theme {
    pub fn detect() -> Self {
        let mode = detect_theme_mode();
        let palette = match mode {
            ThemeMode::Dark => ColorPalette {
                link_fg: Color::Blue,
                inline_code_bg: Color::Rgb(68, 68, 68),
                code_block_bg: Color::Rgb(38, 38, 38),
                blockquote_border: Color::Rgb(100, 100, 100),
                search_highlight: Color::Yellow,
                search_current: Color::Rgb(255, 255, 0),
            },
            ThemeMode::Light => ColorPalette {
                link_fg: Color::DarkGray,
                inline_code_bg: Color::Rgb(252, 252, 252),
                code_block_bg: Color::Rgb(254, 254, 254),
                blockquote_border: Color::Rgb(140, 140, 140),
                search_highlight: Color::Yellow,
                search_current: Color::Rgb(255, 255, 0),
            },
        };
        Self { mode, palette }
    }

    pub fn heading_fg(&self, level: u8) -> Color {
        match self.mode {
            ThemeMode::Dark => match level {
                1 => Color::Indexed(39),  // cyan
                2 => Color::Indexed(76),  // green
                3 => Color::Indexed(178), // orange/yellow
                4 => Color::Indexed(168), // pink/rose
                5 => Color::Indexed(31),  // blue
                6 => Color::Indexed(66),  // muted green
                _ => Color::Indexed(66),
            },
            ThemeMode::Light => match level {
                1 => Color::Indexed(30),  // dark cyan
                2 => Color::Indexed(28),  // dark green
                3 => Color::Indexed(136), // dark yellow
                4 => Color::Indexed(125), // dark magenta
                5 => Color::Indexed(25),  // dark blue
                6 => Color::Indexed(59),  // dark muted green
                _ => Color::Indexed(59),
            },
        }
    }

    pub fn heading_underline_color(&self) -> Color {
        match self.mode {
            ThemeMode::Dark => Color::Indexed(240),
            ThemeMode::Light => Color::Indexed(248),
        }
    }

    pub fn heading_underline_char(level: u8) -> Option<&'static str> {
        match level {
            1 => Some("═"),
            2 => Some("─"),
            3 => Some("┄"),
            4 => Some("· "),
            _ => None, // H5, H6: no underline
        }
    }

    pub fn syntect_theme_name(&self) -> &'static str {
        match self.mode {
            ThemeMode::Dark => "base16-ocean.dark",
            ThemeMode::Light => "base16-ocean.light",
        }
    }
}

fn detect_theme_mode() -> ThemeMode {
    // Check COLORFGBG environment variable
    if let Ok(val) = std::env::var("COLORFGBG") {
        if let Some(bg_str) = val.rsplit(';').next() {
            if let Ok(bg) = bg_str.trim().parse::<u8>() {
                return if bg < 8 {
                    ThemeMode::Dark
                } else {
                    ThemeMode::Light
                };
            }
        }
    }
    // Default to dark
    ThemeMode::Dark
}
