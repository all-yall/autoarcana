mod app;
mod ui;
mod event;
mod tui;

use app::App;
use log::info;

use std::sync::mpsc;

use color_eyre::Result;

use crate::engine::player::PlayerID;

use tokio::sync::broadcast::Receiver as BroadcastReceiver;

/// Stand in for an actual type that the server can send us
#[derive(Clone)]
pub struct GameStateSnapshot;

pub struct Client {
    player_id: PlayerID,
    request: mpsc::Sender<PlayerActionRequest>,
    response: mpsc::Receiver<PlayerActionResponse>,
}

pub type PlayerActionRequest = Vec<PlayerAction>;
pub type PlayerActionResponse = PlayerAction;

#[derive(Debug)]
pub enum PlayerAction {
    Pass,
    CardPlay(usize, String),
}

impl Client {
    pub fn launch(player_id: PlayerID, state_update_receiver: BroadcastReceiver<GameStateSnapshot>) -> Result<Client> {
        info!("Launching client for {player_id:?}");

        let (request_sender, request_receiver) = mpsc::channel();
        let (response_sender, response_receiver) = mpsc::channel();


        std::thread::spawn(move || {
            let mut app = App::new(player_id, state_update_receiver, request_receiver, response_sender);
            app.run().expect("app failed to run")
        });

        let client = Client {
            player_id,
            request: request_sender,
            response: response_receiver,
        };
        Ok(client)

    }

    pub fn choose_options(&mut self, options: PlayerActionRequest) -> PlayerActionResponse {
        log::trace!("Requesting player decision from player {:?}, options presented are: {:?}", self.player_id, &options);
        self.request.send(options).expect("could not send request to client");
        match self.response.recv() {
            Ok(resp) => resp,
            Err(e) => {
                log::error!("Failed to connect to client with player id {:?}: {}", self.player_id, e);
                todo!("send a dummy response")
            }
        }
    }
}
