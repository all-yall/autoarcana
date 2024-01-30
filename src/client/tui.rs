use std::{io, panic};

use color_eyre::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

pub type CrosstermTerminal =
    ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stderr>>;

use crate::client::{app::App, event::EventHandler, ui};

/// Representation of terminal user interface.
/// Repsonible for setting up terminal,
/// initializing the interface and handling draw events.
pub struct Tui {
    terminal: CrosstermTerminal,
    pub events: EventHandler,
}

impl Tui {
    /// Creates a new [`Tui`]
    pub fn new(terminal: CrosstermTerminal, events: EventHandler) -> Self {
        Self { terminal, events }
    }

    /// Initializes the terminal interface
    pub fn enter(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(
            io::stderr(),
            EnterAlternateScreen,
            EnableMouseCapture
        )?;

        // Define a custom panic hook to reset the terminal properties
        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            Self::reset().expect("failed to reset the terminal");
            panic_hook(panic_info);
        }));

        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    /// Exits the terminal interface.
    pub fn exit(&mut self) -> Result<()> {
        Self::reset()?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    /// Resets the terminal interface.
    ///
    /// This function is also used for the panic hook to revert terminal properties.
    pub fn reset() -> Result<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(
            io::stderr(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        Ok(())
    }


    /// [`Draw`] the terminal interface by [`rendering`] the widgets.
    /// 
    /// [`Draw`]: tui::Terminal::draw
    /// [`rendering`]: crate::client::ui::render
    pub fn draw(&mut self, app: &mut App) -> Result<()> {
        self.terminal.draw(|frame| ui::render(app, frame))?;
        Ok(())
    }
}



