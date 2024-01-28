mod engine;
mod impls;

use impls::cards::get_card;
use engine::game::Game;


fn main() {
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
