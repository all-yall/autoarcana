mod engine;
mod impls;
mod client;

use impls::cards::get_card;
use engine::game::Game;
use log::{info, LevelFilter};


fn main() {
    // initialize logging
    simple_logging::log_to_file("test.log", LevelFilter::Info).expect("failed to initialize logger");

    info!("Entering autoarcana serverside");

    let cards = || vec![
        get_card("mountain"),
        get_card("mountain"),
        get_card("mountain"),
        get_card("mountain"),
        get_card("miraris wake"),
        get_card("miraris wake"),
        get_card("goblin assailant"),
        get_card("goblin assailant"),
    ];
    let mut game = Game::new(vec![cards(), cards()]);
    game.run();
}
