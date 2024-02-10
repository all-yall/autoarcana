mod app;
mod ui;
mod event;
mod tui;
mod input_handler;
mod game_state_listener;
mod player_action_listener;

use app::App;
use log::info;

use std::sync::mpsc;

use color_eyre::Result;

use crate::{
    engine::{
        player::PlayerID,
        prelude::CardID,
        ability::{AbilityID, AssignedAbility}, card_play::AssignedCardPlay,
    },
    client::{
        player_action_listener::PlayerActionListener,
        game_state_listener::GameStateListener
    }
};

use tokio::sync::broadcast::Receiver as BroadcastReceiver;

/// Stand in for an actual type that the server can send us
#[derive(Clone)]
pub struct GameStateSnapshot;

pub struct Client {
    player_id: PlayerID,
    request: mpsc::Sender<PlayerActionRequest>,
    response: mpsc::Receiver<PlayerActionResponse>,
}

/// A `PlayerActionRequest` is a list of available actions
/// that are legal for the applicable player to take.
/// The first element is reserved for a default or no-op action.
pub type PlayerActionRequest = Vec<PlayerAction>;
/// A `PlayerActionResponse` is among the list of available actions, or is PlayerAction::Pass
pub type PlayerActionResponse = PlayerAction;

#[derive(Debug, Clone)]
pub enum PlayerAction {
    Pass,
    CardPlay(AssignedCardPlay, String),
    ActivateAbility(AssignedAbility, String),
}

impl Client {
    pub fn launch(player_id: PlayerID, state_update_receiver: BroadcastReceiver<GameStateSnapshot>) -> Result<Client> {
        info!("Launching client for {player_id:?}");

        let (request_sender, request_receiver) = mpsc::channel();
        let (response_sender, response_receiver) = mpsc::channel();

        let game_state_listener = GameStateListener::from(state_update_receiver);
        let player_asks = PlayerActionListener::from(request_receiver, response_sender);

        std::thread::spawn(move || {
            let mut app = App::new(player_id, game_state_listener, player_asks);
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
        log::info!("Requesting player decision from player {:?}, options presented are: {:?}", self.player_id, &options);
        self.request.send(options)
            .map_err(|e| {
                log::error!("Could not send request to client: {e}");
                e
            })
            .expect("could not send request to client");
        log::info!("Blocking on player response");
        match self.response.recv() {
            Ok(resp) => resp,
            Err(e) => {
                log::error!("Failed to connect to client with player id {:?}: {}", self.player_id, e);
                PlayerAction::Pass
            }
        }
    }
}
