use color_eyre::Result;

use super::event::EventHandler;

pub enum InputEvent {

}
pub struct InputHandler {
    events: EventHandler,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            events: EventHandler::new(),
        }
    }
    pub fn next(&mut self) -> Result<InputEvent> {
        self.events.next().unwrap()
    }
}
