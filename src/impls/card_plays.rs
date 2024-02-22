use crate::engine::prelude::*;

pub struct CastSpell {}
impl CastSpell {
    pub fn new() -> Box<Self> {
        Box::new(Self{})
    }
}

impl Spawner for CastSpell {
    fn spawn(&self, card_id: CardID, game: &Game) -> Object {
        let new_perm_id = game.perm_ids.get_id();
        let card = game.get(card_id);
        let permanent = Permanent::from_card(card, new_perm_id, card.owner);
        permanent.into()
    }

    fn cost(&self, card_id: CardID, game: &Game) -> Cost {
        game.get(card_id).attrs.cost.clone()
            .map(|mana_cost| Cost::empty().with_mana(mana_cost))
            .unwrap_or_else(Cost::empty)
    }
}

pub fn def_card_plays(card: &mut LatentCard) {
    let speed = if card.attributes.type_line.is(CardType::Instant) {
        AbilitySpeed::Instant
    } else {
        AbilitySpeed::Sorcery
    };
    
    card.card_plays.push(
        CardPlay::new(CastSpell::new(), format!("{}", card.attributes.name), speed)
    );
}
