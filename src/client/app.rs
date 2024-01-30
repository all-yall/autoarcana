use std::sync::mpsc;
use std::time::Duration;

use crate::engine::player::PlayerID;

use super::{GameStateSnapshot, PlayerActionRequest, PlayerActionResponse};
use super::tui::Tui;
use super::event::{Event, EventHandler};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use color_eyre::Result;
use log::info;
use tokio::sync::broadcast::Receiver as BroadcastReceiver;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Mode {
    #[default]
    Running,
    Quitting
}

/// The terminal application
pub struct App {
    /// Channel to receive game updates from server
    game_update: BroadcastReceiver<GameStateSnapshot>,
    /// Game state as we last received it.
    game_state: Option<GameStateSnapshot>,

    /// The player ID of our user
    playerID: PlayerID,


    /// Channel to receive requests from the server for the player to decide something
    requests: mpsc::Receiver<PlayerActionRequest>,
    /// Represents the current request we are displaying to the user
    curr_request: Option<PlayerActionRequest>,
    /// Channel to send response back to the server.
    response: mpsc::Sender<PlayerActionResponse>,

    /// The state of the application.
    mode: Mode,
}

impl App {
    /// Constructs a new instance of [`App`]
    pub fn new(playerID: PlayerID, game_update: BroadcastReceiver<GameStateSnapshot>, requests: mpsc::Receiver<PlayerActionRequest>, response: mpsc::Sender<PlayerActionResponse>) -> Self {
        Self {
            game_update,
            game_state: None,
            playerID,
            requests,
            curr_request: None,
            response,
            mode: Mode::Running,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        info!("Starting clientside application");

        info!("Initializing backend terminal");
        let backend = ratatui::backend::CrosstermBackend::new(std::io::stderr());

        info!("Initializing ratatui terminal");
        let terminal = ratatui::Terminal::new(backend)?;

        info!("Configuring user event handler");
        let events = EventHandler::new();
        let mut tui = Tui::new(terminal, events);
        tui.enter()?;

        let timeout = Duration::from_millis(400);
        info!("Entering main loop");
        while self.running() {
            // draw to screen
            tui.draw(self)?;

            // handle player input
            if let Some(event) = tui.events.next(timeout)? {
                info!("Received user input: {event:?}");
                match event {
                    Event::Tick => self.tick(),
                    Event::Key(key_event) => self.update(key_event),
                    Event::Mouse(_) => {},
                    Event::Resize(_, _) => {},
                }
            };

            // handle game updates from server
            match self.game_update.try_recv()  {
                Ok(state) => {
                    info!("Received game update");
                    self.game_state = Some(state);
                }
                Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {
                    // no update: keep waiting
                },
                Err(e) => return Err(e.into()),
            }

            // handle requests for the player to make a decision
            // this is skipped if we already are working on a request
            if self.curr_request.is_none() {
                match self.requests.try_recv() {
                    Ok(request) => {
                        info!("Received request for player to make a decision");
                        self.curr_request = Some(request);
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        // nothing received, try again
                    }
                    Err(e) => return Err(e.into()),
                }
            }

        }

        // exit the user interface
        info!("Tearing down user interface");
        tui.exit()?;

        Ok(())
    }

    pub fn running(&self) -> bool {
        self.mode != Mode::Quitting
    }

    /// Handles the tick event of the terminal
    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.mode = Mode::Quitting;
    }

    pub fn update(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.quit(),
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    self.quit()
                }
            }
            _ => {},
        }
    }
}

/*
impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ]);
        let [title_bar, tab, bottom_bar] = area.split(&vertical);

        Block::new().render(area, buf);
    }
}
*/

