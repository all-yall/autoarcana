use std::sync::mpsc;

use crate::engine::player::PlayerID;

use super::{GameStateSnapshot, PlayerActionRequest, PlayerActionResponse};
use super::tui::Tui;
use super::event::{Event, EventHandler};

use crossterm::event::KeyCode;
use color_eyre::Result;
use log::info;
use tokio::sync::broadcast::Receiver as BroadcastReceiver;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Running,
    Quitting
}

/// The terminal application
pub struct App {
    /// Channel to receive game updates from server
    game_update: BroadcastReceiver<GameStateSnapshot>,
    /// Game state as we last received it.
    pub game_state: Option<GameStateSnapshot>,

    /// The player/// Channel to receive requests from the server for the player to decide something
    requests: mpsc::Receiver<PlayerActionRequest>,
    /// Represents the current request we are displaying to the user
    pub curr_request: Option<PlayerActionRequest>,
    /// Channel to send response back to the server.
    response: mpsc::Sender<PlayerActionResponse>,
    /// A possible response to the player action request we have received.
    pub pending_response: Option<PlayerActionResponse>,

    /// ID of our user
    pub player_id: PlayerID,

    /// Keys that were pressed down the previous frame
    key_down: Vec<KeyCode>,
    
    /// The state of the application.
    mode: Mode,
}

impl App {
    /// Constructs a new instance of [`App`]
    pub fn new(player_id: PlayerID, game_update: BroadcastReceiver<GameStateSnapshot>, requests: mpsc::Receiver<PlayerActionRequest>, response: mpsc::Sender<PlayerActionResponse>) -> Self {
        Self {
            game_update,
            game_state: None,
            player_id,
            requests,
            curr_request: None,
            response,
            pending_response: None,
            mode: Mode::Running,
            key_down: vec![],
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

        info!("Entering main loop");
        while self.running() {
            // draw to screen
            tui.draw(self)?;

            self.update();
            
            // clear state
            self.key_down.clear();

            // handle player input
            if let Some(event) = tui.events.try_recv()? {
                info!("Received user input: {event:?}");
                match event {
                    Event::Key(key_event) => {
                        self.key_down.push(key_event.code);
                    }
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
            
            // if we have a response to the server's request, then send it
            if self.curr_request.is_some() {
                if let Some(resp) = self.pending_response.clone() {
                    self.response.send(resp).expect("could not send response to server");
                    self.pending_response = None;
                    self.curr_request = None;
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

    pub fn quit(&mut self) {
        self.mode = Mode::Quitting;
    }

    pub fn is_key_down(&mut self, key: KeyCode) -> bool {
        self.key_down.contains(&key)
    }

    pub fn update(&mut self) {
        if self.is_key_down(KeyCode::Esc) {
            self.quit();
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

