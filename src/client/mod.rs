mod app;
mod ui;
mod event;
mod tui;

use app::App;

use std::sync::mpsc;

use color_eyre::Result;

use crate::engine::player::PlayerID;

use tokio::sync::broadcast::Receiver as BroadcastReceiver;

/// Stand in for an actual type that the server can send us
#[derive(Clone)]
pub struct GameStateSnapshot;

pub struct Client {
    request: mpsc::Sender<PlayerActionRequest>,
    response: mpsc::Receiver<PlayerActionResponse>,
}

pub type PlayerActionRequest = Vec<PlayerAction>;
pub type PlayerActionResponse = PlayerAction;

pub enum PlayerAction {
    Pass,
    CardPlay(usize, String),
}

impl Client {
    pub fn launch(playerID: PlayerID, state_update_receiver: BroadcastReceiver<GameStateSnapshot>) -> Result<Client> {

        let (request_sender, request_receiver) = mpsc::channel();
        let (response_sender, response_receiver) = mpsc::channel();


        std::thread::spawn(move || {
            let mut app = App::new(playerID, state_update_receiver, request_receiver, response_sender);
            app.run().expect("app failed to run")
        });

        let client = Client {
            request: request_sender,
            response: response_receiver,
        };
        Ok(client)

    }

    pub fn choose_options(&mut self, options: PlayerActionRequest) -> PlayerActionResponse {
        self.request.send(options);

        let resp = self.response.recv().expect("could not receive player action response");

        resp
    }
}
