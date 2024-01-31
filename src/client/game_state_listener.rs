use tokio::sync::broadcast::Receiver;
use log::info;
use color_eyre::Result;

use super::GameStateSnapshot;

pub struct GameStateListener {
    receiver: Receiver<GameStateSnapshot>,
    state: Option<GameStateSnapshot>,
}

impl GameStateListener {
    pub fn from(receiver: Receiver<GameStateSnapshot>) -> Self {
        Self {
            receiver,
            state: None
        }
    }
    pub fn update(&mut self) -> Result<()> {
        // handle game updates from server
        match self.receiver.try_recv()  {
            Ok(state) => {
                info!("Received game update");
                self.state = Some(state);
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {
                // no update: keep waiting
            },
            Err(e) => {
                log::error!("Failed to receive updates from server: {e}");
                return Err(e.into())
            }
        };
        Ok(())
    }
}
