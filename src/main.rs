mod app;
mod event;
mod markdown;
mod search;
mod theme;
mod ui;

use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use clap::Parser as ClapParser;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use notify::Watcher;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout};
use ratatui::Terminal;

use app::{App, InputSource, Mode};
use event::{AppEvent, EventReader};
use theme::Theme;
use ui::error_popup::ErrorPopup;
use ui::search_bar::SearchBar;
use ui::status_bar::StatusBar;
use ui::viewer::ViewerWidget;

use ratatui::widgets::Widget;

#[derive(ClapParser, Debug)]
#[command(name = "mdview", about = "Terminal Markdown Viewer")]
struct Cli {
    /// Markdown file to view
    file: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let theme = Theme::detect();

    // Determine input source
    let (input_source, stdin_content) = if !io::stdin().is_terminal() {
        let mut content = String::new();
        io::stdin().read_to_string(&mut content)?;
        (InputSource::Stdin, Some(content))
    } else if let Some(ref path) = cli.file {
        (InputSource::FileArg(path.clone()), None)
    } else {
        (InputSource::Picker, None)
    };

    let mut app = App::new(input_source.clone(), theme);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let terminal_width = terminal.size()?.width;

    // Setup event reader (must happen before loading content so highlighting works)
    let (mut event_reader, event_tx) = EventReader::new();
    app.highlight_tx = Some(event_tx.clone());

    // Load initial content
    match &input_source {
        InputSource::FileArg(path) => {
            if let Err(e) = app.load_file(path, terminal_width) {
                app.error_message = Some(format!("{e}"));
            }
        }
        InputSource::Stdin => {
            if let Some(ref content) = stdin_content {
                app.load_content(content, "<stdin>", terminal_width);
            }
        }
        InputSource::Picker => {}
    }

    // Setup file watcher
    let _watcher = if matches!(input_source, InputSource::FileArg(_)) {
        setup_file_watcher(&app, event_tx.clone())
    } else {
        None
    };

    // Main loop
    loop {
        // Draw
        terminal.draw(|frame| {
            let area = frame.area();
            app.viewport_height = area.height.saturating_sub(2);

            match app.mode {
                Mode::FilePicker => {
                    let [main_area, status_area] = Layout::vertical([
                        Constraint::Min(1),
                        Constraint::Length(1),
                    ])
                    .areas(area);

                    if let Some(ref mut picker) = app.file_picker {
                        picker.render(main_area, frame.buffer_mut());
                    }

                    let status = StatusBar {
                        filename: "",
                        line_count: 0,
                        word_count: 0,
                        scroll_percent: 0,
                        mode: &app.mode,
                        table_wrap: app.table_wrap,
                        search_info: None,
                    };
                    status.render(status_area, frame.buffer_mut());
                }
                Mode::Viewer | Mode::Search => {
                    let has_search_bar = app.mode == Mode::Search;
                    let bottom_height = if has_search_bar { 2 } else { 1 };

                    let [main_area, bottom_area] = Layout::vertical([
                        Constraint::Min(1),
                        Constraint::Length(bottom_height),
                    ])
                    .areas(area);

                    let link_line_indices = app.link_line_indices();
                    let viewer = ViewerWidget::new(
                        &app.lines,
                        app.scroll_offset,
                        &app.search,
                        app.focused_link,
                        &link_line_indices,
                    );
                    viewer.render(main_area, frame.buffer_mut());

                    if has_search_bar {
                        let [search_area, status_area] = Layout::vertical([
                            Constraint::Length(1),
                            Constraint::Length(1),
                        ])
                        .areas(bottom_area);

                        let search_bar = SearchBar {
                            query: &app.search.query,
                        };
                        search_bar.render(search_area, frame.buffer_mut());

                        let status = StatusBar {
                            filename: &app.filename,
                            line_count: app.lines.len(),
                            word_count: app.word_count,
                            scroll_percent: app.scroll_percent(),
                            mode: &app.mode,
                            table_wrap: app.table_wrap,
                            search_info: app.search.match_info(),
                        };
                        status.render(status_area, frame.buffer_mut());
                    } else {
                        let status = StatusBar {
                            filename: &app.filename,
                            line_count: app.lines.len(),
                            word_count: app.word_count,
                            scroll_percent: app.scroll_percent(),
                            mode: &app.mode,
                            table_wrap: app.table_wrap,
                            search_info: app.search.match_info(),
                        };
                        status.render(bottom_area, frame.buffer_mut());
                    }
                }
            }

            // Error popup overlay
            if let Some(ref msg) = app.error_message {
                let popup = ErrorPopup { message: msg };
                popup.render(area, frame.buffer_mut());
            }
        })?;

        if app.should_quit {
            break;
        }

        // Handle events
        if let Some(event) = event_reader.next().await {
            match event {
                AppEvent::Key(key) => app.handle_key(key),
                AppEvent::Mouse(mouse) => app.handle_mouse(mouse),
                AppEvent::Resize(w, _h) => {
                    if app.mode == Mode::Viewer || app.mode == Mode::Search {
                        if let Some(ref path) = app.current_file.clone() {
                            let _ = app.load_file(path, w);
                        }
                    }
                }
                AppEvent::FileChanged => {
                    if let Some(ref path) = app.current_file.clone() {
                        let width = terminal.size()?.width;
                        let old_scroll = app.scroll_offset;
                        if let Err(e) = app.load_file(path, width) {
                            app.error_message = Some(format!("Reload failed: {e}"));
                        }
                        let max = ui::viewer::max_scroll(app.lines.len(), app.viewport_height);
                        app.scroll_offset = old_scroll.min(max);
                    }
                }
                AppEvent::HighlightDone(result) => {
                    app.apply_highlight(result);
                }
                AppEvent::Tick => {}
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn setup_file_watcher(
    app: &App,
    tx: tokio::sync::mpsc::UnboundedSender<AppEvent>,
) -> Option<notify::RecommendedWatcher> {
    let path = app.current_file.as_ref()?.clone();

    let event_tx = tx;
    let mut last_event = std::time::Instant::now();

    let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
        if let Ok(event) = res {
            if event.kind.is_modify() {
                let now = std::time::Instant::now();
                if now.duration_since(last_event) > Duration::from_millis(300) {
                    last_event = now;
                    let _ = event_tx.send(AppEvent::FileChanged);
                }
            }
        }
    })
    .ok()?;

    watcher
        .watch(&path, notify::RecursiveMode::NonRecursive)
        .ok()?;

    Some(watcher)
}
