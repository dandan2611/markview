use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::time::Duration;
use tokio::sync::mpsc;

use crate::markdown::highlight::HighlightResult;

#[derive(Debug)]
pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    FileChanged,
    HighlightDone(HighlightResult),
    #[allow(dead_code)]
    Tick,
}

pub struct EventReader {
    rx: mpsc::UnboundedReceiver<AppEvent>,
}

impl EventReader {
    pub fn new() -> (Self, mpsc::UnboundedSender<AppEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let crossterm_tx = tx.clone();

        // Spawn crossterm event reader
        tokio::spawn(async move {
            loop {
                if event::poll(Duration::from_millis(50)).unwrap_or(false) {
                    if let Ok(evt) = event::read() {
                        let app_event = match evt {
                            CrosstermEvent::Key(key) => AppEvent::Key(key),
                            CrosstermEvent::Mouse(mouse) => AppEvent::Mouse(mouse),
                            CrosstermEvent::Resize(w, h) => AppEvent::Resize(w, h),
                            _ => continue,
                        };
                        if crossterm_tx.send(app_event).is_err() {
                            break;
                        }
                    }
                }
            }
        });

        (Self { rx }, tx)
    }

    pub async fn next(&mut self) -> Option<AppEvent> {
        self.rx.recv().await
    }
}
