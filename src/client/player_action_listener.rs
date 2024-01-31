use std::sync::mpsc::{Receiver, Sender, TryRecvError};

use color_eyre::Result;
use log::{info, error};

use super::{PlayerActionResponse, PlayerActionRequest};

pub struct PlayerActionListener {
    receiver: Receiver<PlayerActionRequest>,
    sender: Sender<PlayerActionResponse>,

    current_request: Option<PlayerActionRequest>,
}

impl PlayerActionListener {
    pub fn from(receiver: Receiver<PlayerActionRequest>, sender: Sender<PlayerActionResponse>) -> Self {
        Self {
            receiver,
            sender,
            current_request: None,
        }
    }

    pub fn update(&mut self) -> Result<()> {
        if self.current_request.is_none() {
            match self.receiver.try_recv() {
                Ok(request) => {
                    info!("Received request for player to make a decision: {request:?}");
                    self.current_request = Some(request);
                }
                Err(TryRecvError::Empty) => {
                    // nothing received, try again
                }
                Err(e) => {
                    error!("Failed to receive from player action request: {e}");
                    return Err(e.into())
                }
            }
        }

        Ok(())
    }

    pub fn respond(&mut self, response: PlayerActionResponse) -> Result<()> {
        if self.current_request.is_none() {
            error!("Sending a response when there is no request");
            return Ok(());
        }
        info!("Responding to player decision with {response:?}");
        self.sender.send(response)?;
        self.current_request = None;
        Ok(())
    }
}
