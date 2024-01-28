use super::abilities::*;
use crate::engine::prelude::*;

pub fn get_card(name: &str) -> LatentCard {
    use CardType::*;
    use ManaType::*;
    match name {
        "mountain" => LatentCard::new(
            "Mountain".to_string(),
            "One day, night will come to these mountains.".to_string(), 
            vec![Basic, Land(Red)],
            vec![ 
                LatentAbility {
                    class: AbilityClass::Activated(Cost::new().with_tap()),
                    description: "Add one red mana".to_string(),
                    effect: Effect::OneShot(AddManaEffect::new(Red)),
                },
            ],
            None, None,
        ),

        "miraris wake" => LatentCard::new(
            "Mirari's Wake".to_string(),
            "Even after a false god tore the magic from Dominaria, power still radiated from the Mirari sword that slew her.".to_string(),
            vec![Enchantment],
            vec![
                LatentAbility {
                    class: AbilityClass::Static,
                    description: "Creatures you control get +1/+1.".to_string(),
                    effect: Effect::Continuous(NullEffect::new()),
                },
                LatentAbility {
                    class: AbilityClass::Static,
                    description: "Whenever you tap a land for mana, add one mana of any type that land produced.".to_string(),
                    effect: Effect::Continuous(MiraisMana::new()),
                }
            ],
            None, None,
        ),

        "goblin assailant" => LatentCard::new(
            "Goblin Assailant".to_string(),
            "What he lacks in patience, intelligence, empathy, lucidity, hygiene, ability to follow orders, self-regard, and discernible skills, he makes up for in sheer chaotic violence.".to_string(),
            vec![Creature, Goblin, Warrior],
            vec![],
            Some(2), Some(2),
        ),

        other => panic!("no card named '{}'", other),
    }
}
