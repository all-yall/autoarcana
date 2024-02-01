use super::abilities::*;
use crate::engine::prelude::*;

pub fn get_card(name: &str) -> LatentCard {
    use CardType::*;
    use ManaType::*;
    match name {
        "mountain" => LatentCard::new(
            "Mountain".into(),
            ManaCost::empty(),
            "One day, night will come to these mountains.".into(), 
            vec![Basic, Land(Red)],
            vec![],
            vec![ 
                LatentAbility {
                    class: AbilityClass::Activated(Cost::empty().with_tap(), AddManaEffect::new(Red)),
                    description: "Add one red mana".into(),
                },
            ],
            vec![CastSpell::ability()],
            None, None,
        ),

        "miraris wake" => LatentCard::new(
            "Mirari's Wake".into(),
            ManaCost::empty(),
            "Even after a false god tore the magic from Dominaria, power still radiated from the Mirari sword that slew her.".into(),
            vec![Enchantment],
            vec![],
            vec![
                LatentAbility {
                    class: AbilityClass::Static(NullEffect::new()),
                    description: "Creatures you control get +1/+1.".into(),
                },
                LatentAbility {
                    class: AbilityClass::Triggered(MiraisMana::new()),
                    description: "Whenever you tap a land for mana, add one mana of any type that land produced.".into(),
                }
            ],
            vec![CastSpell::ability()],
            None, None,
        ),

        "goblin assailant" => LatentCard::new(
            "Goblin Assailant".into(),
            ManaCost::empty(),
            "What he lacks in patience, intelligence, empathy, lucidity, hygiene, ability to follow orders, self-regard, and discernible skills, he makes up for in sheer chaotic violence.".into(),
            vec![Creature],
            vec!["Goblin".into(), "Warrior".into()],
            vec![],
            vec![CastSpell::ability()],
            Some(2), Some(2),
        ),

        other => panic!("no card named '{}'", other),
    }
}
