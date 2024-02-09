use crate::engine::player::PlayerID;

use super::game_state_listener::GameStateListener;
use super::player_action_listener::PlayerActionListener;
use super::PlayerActionResponse;
use super::tui::Tui;
use super::event::{Event, InputHandler};

use crossterm::event::KeyCode;
use color_eyre::Result;
use log::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Running,
    Quitting
}

/// The terminal application
pub struct App {
    game_state: GameStateListener,

    player_asks: PlayerActionListener,
    pending_response: Option<PlayerActionResponse>,

    /// ID of our user
    pub player_id: PlayerID,
    
    /// The state of the application.
    mode: Mode,

}

impl App {
    /// Constructs a new instance of [`App`]
    pub fn new(player_id: PlayerID, game_state: GameStateListener, player_asks: PlayerActionListener) -> Self {
        Self {
            player_id,
            game_state,
            player_asks,
            pending_response: None,
            mode: Mode::Running,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        info!("Starting clientside application");

        info!("Initializing backend terminal");
        let backend = ratatui::backend::CrosstermBackend::new(std::io::stderr());

        info!("Initializing ratatui terminal");
        let terminal = ratatui::Terminal::new(backend)?;

        let input_handler = InputHandler::new();

        info!("Configuring user event handler");
        let events = InputHandler::new();
        let mut tui = Tui::new(terminal, events);
        tui.enter()?;

        info!("Entering main loop");
        while self.running() {
            // draw to screen
            tui.draw(self)?;

            self.game_state.update()?;
            self.player_asks.update()?;
            // if we have a response to the server's request, then send it
            if let Some(response) = self.pending_response.take() {
                self.player_asks.respond(response)?;
            }

            let input = input_handler.next()?;
            self.update(input);

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

    pub fn update(&mut self, input: InputEvent) {
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

